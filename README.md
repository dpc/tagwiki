<p align="center">
  <a href="https://travis-ci.org/dpc/tagwiki">
      <img src="https://img.shields.io/travis/dpc/tagwiki/master.svg?style=flat-square" alt="Travis CI Build Status">
  </a>
  <a href="https://crates.io/crates/tagwiki">
      <img src="http://meritbadge.herokuapp.com/tagwiki?style=flat-square" alt="crates.io">
  </a>
  <a href="https://matrix.to/#/!VLOvTiaFrBYAYplQFW:mozilla.org">
    <img src="https://img.shields.io/matrix/rust:mozilla.org.svg?server_fqdn=matrix.org&style=flat-square" alt="#rust matrix channel">
  </a>
  <a href="https://gitter.im/rust-lang/rust">
    <img src="https://img.shields.io/gitter/room/rust-lang/rust.svg?style=flat-square" alt="rust-lang gitter channel">
  </a>
  <br>
</p>


# Tagwiki

Tagwiki is a wiki in which you link to pages by specifing hashtags they contain.

Example: `/tagwiki/help` link will lead to all pages that contain both `#tagwiki` and `#help`.

This allows effortless and self-structuring organization and editing experience,
as the page collection grows and evolves.

### My use-case

I just need a personal wiki, that I can throw random things into,
that I don't have to pre-plan or carefully maintain.

### User facing features and design goals

* browser-based UI,
* uses Markdown for content,
* brutally simple,
* fast,
* excelent support for keyboard navigation,
* keeps pages as markdown files in a directory,

### Under the hood

* Rust, `async/await`

### Feature ideas:

* "journal mode" - for easy note taking
* support public-facing setups (authentication, permissions, and so on)

### Installing & running

Like any Rust program. `cargo install --git https://github.com/dpc/tagwiki`.

To run `tagwiki <markdown_files_directory>`

See `docs` directory for more user-documentation. Pages inside it are
tagwiki content, so you can try `tagwiki ./docs` try things out.
