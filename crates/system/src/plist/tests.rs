pub(super) mod writer;

use std::collections::BTreeMap;
use std::fmt::Write as _;

use super::{MAX_PLIST, PlistError, read_strings};
use writer::{Object, Writer, dict_of_strings};

const XML_HEADER: &str = concat!(
    r#"<?xml version="1.0" encoding="UTF-8"?>"#,
    "\n",
    r#"<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">"#,
    "\n",
);

/// An XML property list whose top-level object is `body`.
fn xml(body: &str) -> Vec<u8> {
    format!("{XML_HEADER}<plist version=\"1.0\">\n{body}\n</plist>\n").into_bytes()
}

/// An XML property list whose top-level object is a dictionary of strings; the text is used as it is.
fn xml_dict_of_strings(entries: &[(&str, &str)]) -> Vec<u8> {
    let mut body = String::from("<dict>");
    for (key, value) in entries {
        write!(body, "<key>{key}</key><string>{value}</string>").unwrap();
    }
    body.push_str("</dict>");
    xml(&body)
}

fn map(entries: &[(&str, &str)]) -> BTreeMap<String, String> {
    entries.iter().map(|(key, value)| ((*key).to_owned(), (*value).to_owned())).collect()
}

fn malformed(bytes: &[u8]) -> bool {
    matches!(read_strings(bytes), Err(PlistError::Malformed(_)))
}

/// Overwrites the big-endian trailer field that starts `from_end` bytes before the end.
fn set_trailer(bytes: &mut [u8], from_end: usize, value: &[u8]) {
    let start = bytes.len() - from_end;
    bytes[start..start + value.len()].copy_from_slice(value);
}

/// Replaces the one occurrence of `from` with `to`, of the same length.
fn replace(bytes: &mut [u8], from: &[u8], to: &[u8]) {
    let at = bytes.windows(from.len()).position(|window| window == from).unwrap();
    bytes[at..at + to.len()].copy_from_slice(to);
}

const OFFSET_SIZE: usize = 26;
const REF_SIZE: usize = 25;
const NUM_OBJECTS: usize = 24;
const TOP_OBJECT: usize = 16;
const TABLE_OFFSET: usize = 8;

/// What the system's own writer (`plutil -convert binary1`) produced for a small Info.plist written for this test:
/// UTF-16 and long strings beside an array, an integer, a date, data and a real.
const SYSTEM_WRITTEN_BINARY: &str = concat!(
    "62706c6973743030d90102030405060708090a0b0c0d0e0f1011125f101a434642756e646c6553686f727456657273696f6e537472696e675f101243",
    "4642756e646c654964656e746966696572594c6f6e6756616c756554426c6f6255526174696f5c434642756e646c654e616d65545768656e55436f75",
    "6e745f1015434642756e646c65446f63756d656e74547970657355322e342e315f1012636f6d2e6578616d706c652e73616d706c655f102961207661",
    "6c75652074686174206973206c6f6e676572207468616e206669667465656e20627974657343000102233ff80000000000006f101500530061006d00",
    "70006c00650020002600200043006f00202014002000dc006e00ef0063006f006400653341c782f84000000012000493e0a113d11415517809000800",
    "1b0038004d0057005c0062006f0074007a0092009800ad00d900dd00e60113011c012101230126012800000000000002010000000000000016000000",
    "00000000000000000000000129",
);

fn from_hex(hex: &str) -> Vec<u8> {
    hex.as_bytes().chunks(2).map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap()).collect()
}

const INFO_PLIST_XML: &str = r"<dict>
	<key>CFBundleDevelopmentRegion</key>
	<string>en</string>
	<key>CFBundleDocumentTypes</key>
	<array>
		<dict>
			<key>CFBundleTypeExtensions</key>
			<array>
				<string>txt</string>
			</array>
			<key>CFBundleTypeName</key>
			<string>Plain Text</string>
			<key>LSIsAppleDefaultForType</key>
			<true/>
		</dict>
	</array>
	<key>CFBundleExecutable</key>
	<string>Sample</string>
	<key>CFBundleIdentifier</key>
	<string>com.example.sample</string>
	<key>CFBundleName</key>
	<string>Sample &amp; Co</string>
	<key>CFBundleShortVersionString</key>
	<string>2.4.1</string>
	<key>CFBundleVersion</key>
	<string>2410</string>
	<key>LSMinimumSystemVersion</key>
	<string>13.0</string>
	<key>LSRequiresNativeExecution</key>
	<false/>
	<key>NSHighResolutionCapable</key>
	<true/>
	<key>NSHumanReadableCopyright</key>
	<string>© 2026 Example</string>
</dict>";

fn info_plist_expected() -> BTreeMap<String, String> {
    map(&[
        ("CFBundleDevelopmentRegion", "en"),
        ("CFBundleExecutable", "Sample"),
        ("CFBundleIdentifier", "com.example.sample"),
        ("CFBundleName", "Sample & Co"),
        ("CFBundleShortVersionString", "2.4.1"),
        ("CFBundleVersion", "2410"),
        ("LSMinimumSystemVersion", "13.0"),
        ("NSHumanReadableCopyright", "© 2026 Example"),
    ])
}

/// The same Info.plist in the binary form, written as the system writer lays it out.
fn info_plist_binary() -> Vec<u8> {
    let mut writer = Writer::default();
    let extensions_key = writer.ascii("CFBundleTypeExtensions");
    let txt = writer.ascii("txt");
    let extensions = writer.add(Object::Array(vec![txt]));
    let type_name_key = writer.ascii("CFBundleTypeName");
    let type_name = writer.ascii("Plain Text");
    let default_key = writer.ascii("LSIsAppleDefaultForType");
    let yes = writer.add(Object::Bool(true));
    let no = writer.add(Object::Bool(false));
    let document_type =
        writer.add(Object::Dict(vec![(extensions_key, extensions), (type_name_key, type_name), (default_key, yes)]));
    let document_types = writer.add(Object::Array(vec![document_type]));
    let mut pairs = Vec::new();
    for (key, value) in [
        ("CFBundleDevelopmentRegion", "en"),
        ("CFBundleExecutable", "Sample"),
        ("CFBundleIdentifier", "com.example.sample"),
        ("CFBundleName", "Sample & Co"),
        ("CFBundleShortVersionString", "2.4.1"),
        ("CFBundleVersion", "2410"),
        ("LSMinimumSystemVersion", "13.0"),
    ] {
        pairs.push((writer.ascii(key), writer.ascii(value)));
    }
    pairs.push((writer.ascii("CFBundleDocumentTypes"), document_types));
    pairs.push((writer.ascii("LSRequiresNativeExecution"), no));
    pairs.push((writer.ascii("NSHighResolutionCapable"), yes));
    pairs.push((writer.ascii("NSHumanReadableCopyright"), writer.utf16("© 2026 Example")));
    let top = writer.add(Object::Dict(pairs));
    writer.finish(top)
}

mod negative {
    use super::*;

    #[test]
    fn empty_input_is_not_a_plist() {
        assert_eq!(read_strings(b""), Err(PlistError::NotAPlist));
    }

    #[test]
    fn plain_text_is_not_a_plist() {
        assert_eq!(read_strings(b"hello <plist>"), Err(PlistError::NotAPlist));
        assert_eq!(read_strings(b"\xEF\xBB\xBF  \n"), Err(PlistError::NotAPlist));
        assert_eq!(read_strings(b"bplist01"), Err(PlistError::NotAPlist));
    }

    #[test]
    fn input_over_one_mebibyte_is_too_large() {
        let mut bytes = dict_of_strings(&[("a", "b")]);
        bytes.resize(MAX_PLIST + 1, b' ');
        assert_eq!(read_strings(&bytes), Err(PlistError::TooLarge));
    }

    #[test]
    fn a_top_level_array_is_not_a_dictionary() {
        let mut writer = Writer::default();
        let item = writer.ascii("item");
        let top = writer.add(Object::Array(vec![item]));
        assert_eq!(read_strings(&writer.finish(top)), Err(PlistError::NotADictionary));
        assert_eq!(read_strings(&xml("<array><string>item</string></array>")), Err(PlistError::NotADictionary));
    }

    #[test]
    fn a_top_level_string_is_not_a_dictionary() {
        let mut writer = Writer::default();
        let top = writer.ascii("alone");
        assert_eq!(read_strings(&writer.finish(top)), Err(PlistError::NotADictionary));
        assert_eq!(read_strings(&xml("<string>alone</string>")), Err(PlistError::NotADictionary));
        assert_eq!(read_strings(&xml("<true/>")), Err(PlistError::NotADictionary));
    }

    #[test]
    fn a_binary_header_without_a_trailer_is_malformed() {
        assert!(malformed(b"bplist00"));
        assert!(malformed(&[b"bplist00".as_slice(), &[0; 32]].concat()));
    }

    #[test]
    fn a_truncated_binary_trailer_is_malformed() {
        let bytes = dict_of_strings(&[("a", "b")]);
        assert!(malformed(&bytes[..bytes.len() - 1]));
        assert!(malformed(&bytes[..bytes.len() - 20]));
    }

    #[test]
    fn an_offset_or_ref_size_of_zero_or_nine_is_malformed() {
        for (field, size) in [(OFFSET_SIZE, 0), (OFFSET_SIZE, 9), (REF_SIZE, 0), (REF_SIZE, 9)] {
            let mut bytes = dict_of_strings(&[("a", "b")]);
            set_trailer(&mut bytes, field, &[size]);
            assert!(malformed(&bytes), "field {field} size {size}");
        }
    }

    #[test]
    fn an_offset_table_outside_the_data_is_malformed() {
        for table in [0_u64, 4, 1 << 40, u64::MAX] {
            let mut bytes = dict_of_strings(&[("a", "b")]);
            set_trailer(&mut bytes, TABLE_OFFSET, &table.to_be_bytes());
            assert!(malformed(&bytes), "table at {table}");
        }
    }

    #[test]
    fn an_offset_table_running_into_the_trailer_is_malformed() {
        let mut bytes = dict_of_strings(&[("a", "b")]);
        set_trailer(&mut bytes, OFFSET_SIZE, &[8]);
        assert!(malformed(&bytes));
    }

    #[test]
    fn an_object_count_of_zero_or_larger_than_the_data_is_malformed() {
        for count in [0_u64, 10_000, u64::MAX] {
            let mut bytes = dict_of_strings(&[("a", "b")]);
            set_trailer(&mut bytes, NUM_OBJECTS, &count.to_be_bytes());
            assert!(malformed(&bytes), "count {count}");
        }
    }

    #[test]
    fn a_top_object_beyond_the_object_count_is_malformed() {
        let mut bytes = dict_of_strings(&[("a", "b")]);
        set_trailer(&mut bytes, TOP_OBJECT, &3_u64.to_be_bytes());
        assert!(malformed(&bytes));
    }

    #[test]
    fn an_object_offset_outside_the_data_is_malformed() {
        // Objects: "a" at 8, "b" at 10, the dictionary at 12; the offset table follows at 15.
        let mut bytes = dict_of_strings(&[("a", "b")]);
        let table = bytes.len() - 32 - 3;
        for offset in [0, 15, 0xFF] {
            bytes[table + 2] = offset;
            assert!(malformed(&bytes), "offset {offset}");
        }
        let mut bytes = dict_of_strings(&[("a", "b")]);
        bytes[table] = 0xFF;
        assert!(malformed(&bytes));
    }

    #[test]
    fn a_dictionary_count_larger_than_the_data_is_malformed() {
        let mut bytes = dict_of_strings(&[("a", "b")]);
        assert_eq!(bytes[12], 0xD1);
        bytes[12] = 0xDE;
        assert!(malformed(&bytes));
    }

    #[test]
    fn a_huge_following_dictionary_count_is_malformed() {
        let mut writer = Writer::default();
        let pairs = (0..15).map(|index| (writer.ascii(&format!("k{index}")), writer.ascii("v"))).collect();
        let top = writer.add(Object::Dict(pairs));
        let mut bytes = writer.finish(top);
        let dict = bytes.iter().rposition(|&byte| byte == 0xDF).unwrap();
        bytes[dict + 2] = 0xFF;
        assert!(malformed(&bytes));
    }

    #[test]
    fn many_keys_sharing_one_large_string_are_malformed_before_they_are_decoded() {
        let mut writer = Writer::default();
        let shared = writer.ascii(&"A".repeat(20_000));
        let pairs = (0..300).map(|key| (writer.ascii(&format!("k{key:03}")), shared)).collect();
        let top = writer.add(Object::Dict(pairs));
        let bytes = writer.finish(top);

        assert!(bytes.len() < 32 * 1024, "{}", bytes.len());
        assert_eq!(read_strings(&bytes), Err(PlistError::Malformed("the strings decode to far more than the data")));
    }

    #[test]
    fn a_length_that_is_not_an_integer_is_malformed() {
        let mut writer = Writer::default();
        let key = writer.ascii("key");
        let value = writer.ascii("a value of more than fifteen bytes");
        let top = writer.add(Object::Dict(vec![(key, value)]));
        let mut bytes = writer.finish(top);
        let marker = bytes.iter().position(|&byte| byte == 0x5F).unwrap();
        bytes[marker + 1] = 0x50;
        assert!(malformed(&bytes));
        bytes[marker + 1] = 0x14;
        assert!(malformed(&bytes));
    }

    #[test]
    fn a_reference_beyond_the_object_count_is_malformed() {
        let mut bytes = dict_of_strings(&[("a", "b")]);
        bytes[13] = 9;
        assert!(malformed(&bytes));
        let mut bytes = dict_of_strings(&[("a", "b")]);
        bytes[14] = 9;
        assert!(malformed(&bytes));
    }

    #[test]
    fn a_string_running_past_the_objects_is_malformed() {
        let mut bytes = dict_of_strings(&[("a", "b")]);
        bytes[10] = 0x5E;
        assert!(malformed(&bytes));
        bytes[10] = 0x6E;
        assert!(malformed(&bytes));
    }

    #[test]
    fn an_xml_unclosed_tag_is_malformed() {
        let cases = [
            "<?xml version=\"1.0\"?><plist><dict><key>a</key><string>b</string>",
            "<plist><dict><key>a</key><string>b</string></dict>",
            "<plist><dict><key>a</key><string>b",
            "<plist><dict><key>a</key><string>b</string></dict></plist",
            "<plist><dict><key>a</key><string",
            "<plist><dict><key>a</key><string <b></string></dict></plist>",
            "<plist><dict><key>a</key><array><string>b</string></dict></plist>",
            "<plist><dict><key>a</key><array>",
            "<plist><dict><key>a",
        ];
        for case in cases {
            assert!(malformed(case.as_bytes()), "{case}");
        }
    }

    #[test]
    fn an_xml_mismatched_close_is_malformed() {
        assert!(malformed(&xml("<dict><key>a</key><string>b</key></dict>")));
        assert!(malformed(&xml("<dict><key>a</key><array><dict></array></dict>")));
        assert!(malformed(&xml("<dict></array>")));
        assert!(malformed(b"<plist></plist>"));
        assert!(malformed(b"<plist/>"));
        assert!(malformed(b"<plist></dict></plist>"));
    }

    #[test]
    fn an_xml_bad_entity_is_malformed() {
        for entity in ["&bogus;", "&amp", "&#xZZ;", "&#x;", "&#;", "&#0;", "&#xD800;", "&#x110000;", "&#12a;", "&;"] {
            let body = format!("<dict><key>a</key><string>x{entity}y</string></dict>");
            assert!(malformed(&xml(&body)), "{entity}");
        }
        assert!(malformed(&xml("<dict><key>&nope;</key><string>v</string></dict>")));
    }

    #[test]
    fn xml_nesting_deeper_than_the_limit_is_malformed() {
        let body = format!("<dict><key>a</key>{}{}</dict>", "<array>".repeat(65), "</array>".repeat(65));
        assert!(malformed(&xml(&body)));
    }

    #[test]
    fn xml_unclosed_markup_is_malformed() {
        assert!(malformed(b"<plist><!-- never closed <dict/></plist>"));
        assert!(malformed(b"<plist><dict><key>a</key><string><![CDATA[x</string></dict></plist>"));
        assert!(malformed(b"<?xml version=\"1.0\""));
        assert!(malformed(b"<!DOCTYPE plist [ <!ENTITY x \"y\"> "));
    }

    #[test]
    fn xml_structure_errors_are_malformed() {
        let bodies = [
            "<dict><string>no key</string></dict>",
            "<dict><key>a</key></dict>",
            "<dict>text</dict>",
            "<dict><key>a</key><string>b<true/></string></dict>",
            "<dict><key>a</key><string><b>x</b></string></dict>",
            "<dict><key>a</key>",
            "<dict><key>a</key><string>b</string>",
            "<dict><key>a</key><string>b</string></dict><dict/>",
            "<dict><key>a</key><string>b</string></dict> trailing",
            "<dict><key>a</key><string>b</string></dict><key/>",
            "<dict><key>a</key>< ></dict>",
            "<dict><key>a</key></ ></dict>",
            "stray<dict/>",
        ];
        for body in bodies {
            assert!(malformed(&xml(body)), "{body}");
        }
        assert!(malformed(b"<plist><dict/></plist><dict/>"));
    }

    #[test]
    fn xml_text_that_is_not_utf8_is_malformed() {
        let mut bytes = b"<plist><dict><key>a</key><string>".to_vec();
        bytes.extend_from_slice(&[0xFF, 0xFE]);
        bytes.extend_from_slice(b"</string></dict></plist>");
        assert!(malformed(&bytes));
    }
}

mod positive {
    use super::*;

    #[test]
    fn keys_sharing_one_value_each_read_it() {
        let mut writer = Writer::default();
        let shared = writer.ascii("Example");
        let pairs = ["CFBundleName", "CFBundleDisplayName", "CFBundleExecutable"]
            .iter()
            .map(|key| (writer.ascii(key), shared))
            .collect();
        let top = writer.add(Object::Dict(pairs));

        let expected =
            map(&[("CFBundleDisplayName", "Example"), ("CFBundleExecutable", "Example"), ("CFBundleName", "Example")]);
        assert_eq!(read_strings(&writer.finish(top)), Ok(expected));
    }

    #[test]
    fn reads_an_xml_dictionary_of_strings() {
        let bytes = xml("<dict>\n\t<key>CFBundleIdentifier</key>\n\t<string>com.example.app</string>\n</dict>");
        assert_eq!(read_strings(&bytes), Ok(map(&[("CFBundleIdentifier", "com.example.app")])));
    }

    #[test]
    fn reads_xml_with_a_byte_order_mark_and_leading_whitespace() {
        let mut bytes = b"\xEF\xBB\xBF \n\t".to_vec();
        bytes.extend_from_slice(&xml("<dict><key>a</key><string>b</string></dict>"));
        assert_eq!(read_strings(&bytes), Ok(map(&[("a", "b")])));
    }

    #[test]
    fn reads_xml_that_starts_with_the_plist_or_doctype_element() {
        let bare = b"<plist version='1.0'><dict><key>a</key><string>b</string></dict></plist>";
        assert_eq!(read_strings(bare), Ok(map(&[("a", "b")])));
        let doctype = b"<!DOCTYPE plist [ <!ENTITY x 'y'> ]><dict><key>a</key><string>b</string></dict>";
        assert_eq!(read_strings(doctype), Ok(map(&[("a", "b")])));
    }

    #[test]
    fn reads_empty_strings_keys_and_dictionaries() {
        let bytes = xml("<dict><key>a</key><string/><key>b</key><string></string><key/><string>c</string></dict>");
        assert_eq!(read_strings(&bytes), Ok(map(&[("", "c"), ("a", ""), ("b", "")])));
        assert_eq!(read_strings(&xml("<dict/>")), Ok(BTreeMap::new()));
        assert_eq!(read_strings(&xml("<dict></dict>")), Ok(BTreeMap::new()));
        assert_eq!(read_strings(&dict_of_strings(&[])), Ok(BTreeMap::new()));
        assert_eq!(read_strings(&dict_of_strings(&[("", "")])), Ok(map(&[("", "")])));
    }

    #[test]
    fn decodes_entities_in_keys_and_strings() {
        let bytes =
            xml("<dict><key>a&amp;b</key><string>&lt;&gt;&quot;&apos;&#65;&#x42;&#X43;&#x1F600; done</string></dict>");
        assert_eq!(read_strings(&bytes), Ok(map(&[("a&b", "<>\"'ABC\u{1F600} done")])));
    }

    #[test]
    fn reads_cdata_and_ignores_comments() {
        let bytes = xml(
            "<!-- head --><dict><!-- a <comment> --><key>a</key><string>x<![CDATA[<raw> &amp;]]>y<!-- c --></string></dict>",
        );
        assert_eq!(read_strings(&bytes), Ok(map(&[("a", "x<raw> &amp;y")])));
    }

    #[test]
    fn skips_nested_dictionaries_and_arrays() {
        let bytes = xml(concat!(
            "<dict><key>nested</key><dict><key>inner</key><string>hidden</string>",
            "<key>deeper</key><array><dict/><array><string>x</string></array></array></dict>",
            "<key>list</key><array><string>y</string><true/></array><key>empty</key><array/>",
            "<key>kept</key><string>yes</string></dict>",
        ));
        assert_eq!(read_strings(&bytes), Ok(map(&[("kept", "yes")])));
    }

    #[test]
    fn skips_nesting_up_to_the_limit() {
        let body = format!(
            "<dict><key>a</key>{}{}<key>b</key><string>c</string></dict>",
            "<array>".repeat(64),
            "</array>".repeat(64)
        );
        assert_eq!(read_strings(&xml(&body)), Ok(map(&[("b", "c")])));
    }

    #[test]
    fn skips_integers_bools_reals_dates_and_data() {
        let bytes = xml(concat!(
            "<dict><key>i</key><integer>1</integer><key>t</key><true/><key>f</key><false />",
            "<key>r</key><real>1.5</real><key>d</key><date>2026-01-01T00:00:00Z</date>",
            "<key>b</key><data>AAEC</data><key>s</key><string>kept</string></dict>",
        ));
        assert_eq!(read_strings(&bytes), Ok(map(&[("s", "kept")])));

        let mut writer = Writer::default();
        let (int_key, int) = (writer.ascii("i"), writer.add(Object::Int(7)));
        let (bool_key, yes) = (writer.ascii("t"), writer.add(Object::Bool(true)));
        let (string_key, string) = (writer.ascii("s"), writer.ascii("kept"));
        let top = writer.add(Object::Dict(vec![(int_key, int), (bool_key, yes), (string_key, string)]));
        assert_eq!(read_strings(&writer.finish(top)), Ok(map(&[("s", "kept")])));
    }

    #[test]
    fn the_last_of_repeated_keys_wins() {
        let bytes = xml("<dict><key>a</key><string>1</string><key>a</key><string>2</string></dict>");
        assert_eq!(read_strings(&bytes), Ok(map(&[("a", "2")])));
        assert_eq!(read_strings(&dict_of_strings(&[("a", "1"), ("a", "2")])), Ok(map(&[("a", "2")])));
    }

    #[test]
    fn reads_a_binary_dictionary_of_strings() {
        let bytes = dict_of_strings(&[("CFBundleIdentifier", "com.example.app"), ("CFBundleVersion", "12")]);
        assert_eq!(
            read_strings(&bytes),
            Ok(map(&[("CFBundleIdentifier", "com.example.app"), ("CFBundleVersion", "12")]))
        );
    }

    #[test]
    fn reads_utf16_keys_and_strings() {
        let mut writer = Writer::default();
        let key = writer.utf16("Ключ");
        let value = writer.utf16("Значение \u{1F600}");
        let top = writer.add(Object::Dict(vec![(key, value)]));
        assert_eq!(read_strings(&writer.finish(top)), Ok(map(&[("Ключ", "Значение \u{1F600}")])));
    }

    #[test]
    fn reads_lengths_that_follow_the_marker() {
        let long = "a value that is longer than fifteen bytes";
        let mut writer = Writer::default();
        let mut pairs = vec![(writer.ascii("long"), writer.ascii(long)), (writer.ascii("wide"), writer.utf16(long))];
        let many: Vec<(String, String)> = (0..20).map(|index| (format!("k{index:02}"), format!("v{index}"))).collect();
        for (key, value) in &many {
            pairs.push((writer.ascii(key), writer.ascii(value)));
        }
        let top = writer.add(Object::Dict(pairs));
        let mut expected: BTreeMap<String, String> = many.into_iter().collect();
        expected.insert("long".to_owned(), long.to_owned());
        expected.insert("wide".to_owned(), long.to_owned());
        assert_eq!(read_strings(&writer.finish(top)), Ok(expected));
    }

    #[test]
    fn reads_two_byte_references_and_offsets() {
        let entries: Vec<(String, String)> =
            (0..200).map(|index| (format!("key{index}"), format!("{index}"))).collect();
        let borrowed: Vec<(&str, &str)> = entries.iter().map(|(key, value)| (key.as_str(), value.as_str())).collect();
        let bytes = dict_of_strings(&borrowed);
        assert_eq!(bytes[bytes.len() - REF_SIZE], 2);
        assert_eq!(bytes[bytes.len() - OFFSET_SIZE], 2);
        assert_eq!(read_strings(&bytes), Ok(entries.into_iter().collect()));
    }

    #[test]
    fn skips_entries_with_a_non_string_key() {
        let mut writer = Writer::default();
        let (int_key, int_value) = (writer.add(Object::Int(1)), writer.ascii("dropped"));
        let (key, value) = (writer.ascii("a"), writer.ascii("b"));
        let top = writer.add(Object::Dict(vec![(int_key, int_value), (key, value)]));
        assert_eq!(read_strings(&writer.finish(top)), Ok(map(&[("a", "b")])));
    }

    #[test]
    fn skips_strings_that_cannot_be_decoded() {
        let mut bytes = dict_of_strings(&[("a", "b"), ("c", "d")]);
        // Objects: "a" 8, "b" 10, "c" 12, "d" 14; "b" is made non-ASCII.
        bytes[11] = 0xE9;
        assert_eq!(read_strings(&bytes), Ok(map(&[("c", "d")])));

        let mut writer = Writer::default();
        let (lone_key, lone) = (writer.ascii("lone"), writer.utf16("\u{1F600}"));
        let (bad_key, bad_key_value) = (writer.utf16("\u{1F601}"), writer.ascii("x"));
        let (key, value) = (writer.ascii("ok"), writer.utf16("fine"));
        let top = writer.add(Object::Dict(vec![(lone_key, lone), (bad_key, bad_key_value), (key, value)]));
        let mut bytes = writer.finish(top);
        // Both emoji lose their low surrogate to an `A`, which leaves a lone high surrogate.
        replace(&mut bytes, &[0xD8, 0x3D, 0xDE, 0x00], &[0xD8, 0x3D, 0x00, b'A']);
        replace(&mut bytes, &[0xD8, 0x3D, 0xDE, 0x01], &[0xD8, 0x3D, 0x00, b'A']);
        assert_eq!(read_strings(&bytes), Ok(map(&[("ok", "fine")])));
    }

    #[test]
    fn xml_and_binary_with_the_same_content_agree() {
        let from_xml = read_strings(&xml(INFO_PLIST_XML));
        let from_binary = read_strings(&info_plist_binary());
        assert_eq!(from_xml, from_binary);
        let small = [("a", "1"), ("b", "two"), ("c", "")];
        assert_eq!(read_strings(&xml_dict_of_strings(&small)), read_strings(&dict_of_strings(&small)));
    }

    #[test]
    fn reads_what_the_system_writer_produced() {
        let expected = map(&[
            ("CFBundleIdentifier", "com.example.sample"),
            ("CFBundleName", "Sample & Co \u{2014} \u{dc}n\u{ef}code"),
            ("CFBundleShortVersionString", "2.4.1"),
            ("LongValue", "a value that is longer than fifteen bytes"),
        ]);
        assert_eq!(read_strings(&from_hex(SYSTEM_WRITTEN_BINARY)), Ok(expected));
    }

    #[test]
    fn reads_a_realistic_app_info_plist() {
        assert_eq!(read_strings(&xml(INFO_PLIST_XML)), Ok(info_plist_expected()));
        assert_eq!(read_strings(&info_plist_binary()), Ok(info_plist_expected()));
    }

    #[test]
    fn reads_input_of_exactly_one_mebibyte() {
        let mut bytes = xml("<dict><key>a</key><string>b</string></dict>");
        bytes.resize(MAX_PLIST, b'\n');
        assert_eq!(read_strings(&bytes), Ok(map(&[("a", "b")])));
    }
}

mod property {
    use proptest::prelude::*;

    use super::*;

    proptest! {
        #[test]
        fn arbitrary_bytes_never_panic(bytes in proptest::collection::vec(any::<u8>(), 0..512)) {
            let _ = read_strings(&bytes);
        }

        #[test]
        fn arbitrary_bytes_after_the_binary_magic_never_panic(tail in proptest::collection::vec(any::<u8>(), 0..512)) {
            let bytes = [b"bplist00".as_slice(), &tail].concat();
            let _ = read_strings(&bytes);
        }

        #[test]
        fn mutations_of_a_valid_binary_plist_never_panic(
            edits in proptest::collection::vec((any::<prop::sample::Index>(), any::<u8>()), 1..8),
        ) {
            let mut bytes = info_plist_binary();
            for (index, value) in edits {
                let at = index.index(bytes.len());
                bytes[at] = value;
            }
            let _ = read_strings(&bytes);
        }

        #[test]
        fn mutations_of_a_valid_xml_plist_never_panic(
            edits in proptest::collection::vec((any::<prop::sample::Index>(), any::<u8>()), 1..8),
        ) {
            let mut bytes = xml(INFO_PLIST_XML);
            for (index, value) in edits {
                let at = index.index(bytes.len());
                bytes[at] = value;
            }
            let _ = read_strings(&bytes);
        }

        #[test]
        fn strings_round_trip_through_both_forms(
            entries in proptest::collection::btree_map("[a-zA-Z0-9 ._-]{0,20}", "[a-zA-Z0-9 ._-]{0,40}", 0..20),
        ) {
            let borrowed: Vec<(&str, &str)> = entries.iter().map(|(key, value)| (key.as_str(), value.as_str())).collect();
            prop_assert_eq!(read_strings(&dict_of_strings(&borrowed)), Ok(entries.clone()));
            prop_assert_eq!(read_strings(&xml_dict_of_strings(&borrowed)), Ok(entries.clone()));
        }
    }
}
