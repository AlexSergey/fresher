import { createContext } from 'react';

// How a path is written on the page: shown, in its tooltip and as its accessible text. As it is, unless the app says
// otherwise (a demo writes the home folder as `~`, so a recording shows no folder of the machine it plays on).
export type PathForm = (path: string) => string;

export const PathFormContext = createContext<PathForm>((path) => path);
