# Contributing

1. Checkout a new feature branch off of `main`:
   ```bash
   git checkout main
   git pull
   git checkout -b feat/my-awesome-feature
   ```
1. Develop.
   - Add unit & integration [tests].
   - Add a [rustdoc] comment to all the things.
1. Test.
   ```bash
   cargo test
   ```
1. Commit; follow the [Conventional Commits] standard.

[conventional commits]: https://conventionalcommits.org/
[tests]: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
[rustdoc]: https://doc.rust-lang.org/rustdoc/what-is-rustdoc.html
[rust]: https://www.rust-lang.org/tools/install
