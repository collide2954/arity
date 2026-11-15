/** The address of the book of arity, of one of its chapters, or of a section of a chapter. */
export function docsLink(chapter?: string, section?: string): string {
  const book = `${import.meta.env.BASE_URL}docs/`;
  if (chapter === undefined) return book;
  return `${book}${chapter}.html${section === undefined ? "" : `#${section}`}`;
}
