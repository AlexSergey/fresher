// A feedback component's one action: a button with its label ("Try again", "Show details").
export type FeedbackAction = {
  readonly label: string;
  readonly onClick: () => void;
};
