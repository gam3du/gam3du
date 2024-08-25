# Where, what, how?

This document describes what conventions are being used in this project, what tools are recommended and where to put which content, artifact or question.

## General discussion

General questions about this project can be placed in [our Discord server](https://discord.gg/fVDpb9wsHy) at any time. Due to the origin of this project we accept questions and answers in English and German.

## Issues

If you feel something could be improved (bugs or feature requests) you can [open an issue on GitHub](https://github.com/gam3du/gam3du/issues). Issues are expected to be written in English - feel free to use an online translation service if needed.

## Documentation

### Architecture

Description of the project's structure and technical documentation shall be placed in this repository under `docs` (next to this document) in order to keep them in sync with the actual implementation.

### Code

Code shall be as self-descriptive as possible. In order to achieve that, avoid using abbreviations in identifiers. There's no need to document private functions and elements with obvious purpose (getters, setters, `new`, ...). Public elements however shall have at least a single line of documentation and a description of failure modes - clippy will help you to remember that :)

### User

Documentation on how to use the project shall go to our [Wiki](https://github.com/gam3du/gam3du/wiki). It has yet to be decided where graphics shall be stored so that they can be changed later.

## Code conventions and style

- The use of clippy is mandatory
- The use of rustfmt is mandatory
- Handling all compile errors before pushing to a public branch is mandatory - it shall always be possible to build those branches.
- Handling all warnings in a timely manner is highly recommended. It is ok to suppress specific warnings with an explanatation and maybe a `TODO` marker
- If you suppress any linter message, add a short description about _why_ this was necessary.
- Make use of `todo!` and `unimplemented!` or add `// TODO [your github name] your message` comments if you know your implementation still has holes but it helps you to make progress.
- Use `// FIXME [your github name] <your message>` if you spot/create any error you cannot or don't want to fix (yet).
