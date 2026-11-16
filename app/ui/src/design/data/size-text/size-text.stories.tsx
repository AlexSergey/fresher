import { stories as storiesOf } from '../../../gallery/story';
import { SizeText } from './size-text';
import styles from './size-text.stories.module.css';

export const stories = storiesOf('SizeText', {
  complete: () => <SizeText accessibleText="42.7 GB" completeness="complete" size="title-3" text="42.7 GB" />,
  partial: () => (
    <SizeText
      accessibleText="at least 42.7 GB"
      completeness="partial"
      hint="Some folders could not be read"
      size="title-3"
      text="42.7 GB"
    />
  ),
  sizes: () => (
    <div className={styles.column}>
      {(['body', 'callout', 'title-3', 'display', 'hero'] as const).map((size) => (
        <SizeText accessibleText="128.4 GB" completeness="complete" key={size} size={size} text="128.4 GB" />
      ))}
    </div>
  ),
  unknown: () => <SizeText accessibleText="Size not known" completeness="unknown" size="title-3" />,
});
