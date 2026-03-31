This project is about building long-term hierarchically organised, cross-linked
global memory for LLMs, that can be added to any given LLM (e.g. Claude Code)
project. The core idea is to use Mastra's Observational Memory concepts to
simulate how senior human developers accumulate knowledge over time, and build
up expertise in certain techniques, domains, tools, languages and specific
codebases. This knowledge needs to be organised along multiple axes and
carefully indexed, because it is intended to be global i.e. over all the
projects a developer works on, and always growing over time. In an LLM context,
it won't be possible to load it all, so the indexing and cross-referencing needs
to be optimised for context preservation.

From an implementation perspective this needs to be done outside of any given
agent harness so it isn't tied to a particular product, model or architecture.

Mnemosyne is the Greek Titaness of memory and remembrance, representing the
preservation of knowledge and history in ancient oral culture. As the daughter
of Uranus and Gaia and mother of the nine Muses by Zeus, she is a crucial deity
of memory and the namesake for modern mnemonic devices.
