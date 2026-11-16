import type { JSX } from 'react';

import { stories as storiesOf } from '../../../gallery/story';
import { SearchField, type SearchFieldProps } from './search-field';

const ignore = (): void => undefined;

const search = (props: Partial<SearchFieldProps>): (() => JSX.Element) =>
  function Story(): JSX.Element {
    return (
      <SearchField
        clearLabel="Clear the search"
        hideLabel
        label="Search findings"
        onChange={ignore}
        placeholder="Search"
        value=""
        {...props}
      />
    );
  };

const WIDTH = 280;

export const stories = storiesOf('SearchField', {
  empty: { render: search({}), width: WIDTH },
  focus: { render: search({ 'data-focus': true, value: 'xcode' }), width: WIDTH },
  'with-label': { render: search({ hideLabel: false }), width: WIDTH },
  'with-text': { render: search({ value: 'xcode' }), width: WIDTH },
});
