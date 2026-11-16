// The interactive states a gallery story forces, since a screenshot cannot hover, press or focus: a primitive passes
// them to its native element, and its stylesheet draws `[data-hover]` as `:hover`, `[data-pressed]` as `:active` and
// `[data-focus]` as `:focus-visible`. The app never sets them.
export type Forced = {
  readonly 'data-focus'?: true;
  readonly 'data-hover'?: true;
  readonly 'data-pressed'?: true;
};
