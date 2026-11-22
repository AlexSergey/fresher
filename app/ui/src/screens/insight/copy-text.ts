// Writes the text to the clipboard; `false` when the clipboard refuses or the app has none.
export const copyText = async (text: string): Promise<boolean> => {
  try {
    await navigator.clipboard.writeText(text);

    return true;
  } catch {
    return false;
  }
};
