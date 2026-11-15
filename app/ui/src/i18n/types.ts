import type { TextArg } from '../ipc/bindings.generated';
import type chrome from '../locales/en.json';

// `Completeness`, `Dictionary`, `Text` and `TextArg` are the core's types, generated from Rust (plan 05/01).
export type { Completeness, Dictionary, Text, TextArg } from '../ipc/bindings.generated';

export type TextArgs = Readonly<Record<string, TextArg>>;

export type UiKey = WithoutPlural<keyof typeof chrome>;

// A key of the UI's own chrome; a plural text `<key>.one` / `<key>.other` is asked for by `<key>`.
type WithoutPlural<Key extends string> = Key extends `${infer Base}.${'one' | 'other'}` ? Base : Key;
