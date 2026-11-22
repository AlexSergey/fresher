//! The scan test of every demo world of `fixtures/demos/`, through the app's own sandbox composition with every
//! module (docs/plans/038-demo-recordings/01-world/03-first-world.md): the world is generated at its own `now`, scanned,
//! and every category of its `shows` must offer something. A world that stops showing what a recording needs fails
//! here, naming the world and the category. Sandbox mode exists in debug builds only, and so do these tests.
//!
//! A new demo world gets its line in `demo_worlds!` below; the guard fails until it has one.

#![cfg(debug_assertions)]

// The sandbox autotest's helpers are reused; `cfg(test)` lets Clippy's in-test allowances apply to them.
#[cfg(test)]
#[expect(dead_code, reason = "the world scan uses a part of the autotest's helpers")]
#[path = "sandboxes/harness.rs"]
mod harness;
#[cfg(test)]
#[path = "sandboxes/insight.rs"]
mod insight;
#[cfg(test)]
#[expect(dead_code, reason = "the world scan uses a part of the autotest's helpers")]
#[path = "sandboxes/left_out.rs"]
mod left_out;
#[cfg(test)]
#[expect(dead_code, reason = "the world scan uses a part of the autotest's helpers")]
#[path = "sandboxes/map.rs"]
mod map;
#[cfg(test)]
#[expect(dead_code, reason = "the world scan uses a part of the autotest's helpers")]
#[path = "sandboxes/trash.rs"]
mod trash;
#[cfg(test)]
#[path = "demos/worlds.rs"]
mod worlds;

/// One `#[test]` per demo world: `test_name => "file name without .toml"`, sorted by file name.
macro_rules! demo_worlds {
    ($($test:ident => $file:literal),* $(,)?) => {
        /// The demo worlds that have a test.
        const DEMO_WORLDS: &[&str] = &[$($file),*];

        $(
            #[test]
            fn $test() {
                crate::worlds::check_demo_world($file);
            }
        )*
    };
}

demo_worlds! {
    developer_mac => "developer-mac",
}

#[test]
fn every_demo_world_has_a_test() {
    let missing: Vec<String> =
        worlds::demo_world_names().into_iter().filter(|name| !DEMO_WORLDS.contains(&name.as_str())).collect();

    assert!(missing.is_empty(), "add a line to `demo_worlds!` for: {}", missing.join(", "));
}
