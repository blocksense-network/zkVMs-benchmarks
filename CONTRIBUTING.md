# Contributing to zkVMs-benchmarks

The goal of zkVMs-benchmarks is to deliver reproducible and accurate performance data accross many zero knowledge virtual machines and many use cases (programs).
We encourage and warmly welcome any contributions, even if you have limited experience with these technologies.

**No contribution is too small and all contributions are valued.**

This document will help you get started. **Do not let the document intimidate you**.
It should be considered as a guide to help you navigate the process.

All contributions will be made under the MIT license.

## Code of Conduct

The zkVMs-benchmarks project adheres to the [Rust Code of Conduct][rust-coc]. This code of conduct describes the _minimum_ behavior
expected from all contributors.

Instances of violations of the Code of Conduct can be reported by contacting the team.

## Ways to contribute

There are fundamentally three ways an individual can contribute:

1. **By opening an issue:** For example, if you believe that you have uncovered a bug,
   creating a new issue in the issue tracker is the way to report it.
2. **By adding context:** Providing additional context to existing issues,
   such as screenshots and code snippets to help resolve issues.
3. **By resolving issues:** Typically this is done in the form of either
   demonstrating that the issue reported is not a problem after all, or more often,
   by opening a pull request that fixes the underlying problem, in a concrete and
   reviewable manner.

**Anybody can participate in any stage of contribution**. We urge you to participate in the discussion around bugs.

### Submitting a bug report

If you believe that you have uncovered a bug, please provide a description to the best of your ability. Do not worry
if you cannot answer every detail, just write what you can. Contributors will ask follow-up questions if something is
unclear.

The most important pieces of information we need in a bug report are:

- The repository version/revision you are on
- The platform you are on (Windows, macOS, an M1 Mac or Linux)
- Code snippets if this is happening in relation to testing or building code
- Concrete steps to reproduce the bug

In order to rule out the possibility of the bug being in your project, the code snippets should be as minimal as
possible. It is better if you can reproduce the bug with a small snippet as opposed to a large program!

See [this guide][mcve] on how to create a minimal, complete, and verifiable example.

### Submitting a feature request

Please include as detailed of an explanation as possible of the feature you would like, adding additional context if
necessary.

If you have examples of other tools that have the feature you are requesting, please include them as well.

### Resolving an issue

Pull requests are the way concrete changes are made to the code, documentation, and dependencies.

Even tiny pull requests, like fixing wording, are greatly appreciated. Before making a large change, it is usually a
good idea to first open an issue describing the change to solicit feedback and guidance. This will increase the
likelihood of the PR getting merged.

If you are working on a larger feature, we encourage you to open up a draft pull request, to make sure that other
contributors are not duplicating work.

#### Commits

It is a recommended best practice to keep your changes as logically grouped as possible within individual commits. There
is no limit to the number of commits any single pull request may have, and many contributors find it easier to review
changes that are split across multiple commits.

That said, if you have a number of commits that are "checkpoints" and don't represent a single logical change, please
squash those together.

Also, make sure your commit name follows the [conventional commits](https://www.conventionalcommits.org/en/v1.0.0/)
standard. Consistent and easy to parse commits lead to an easier time for maintainers to review your changes!

#### Discuss and update

You will probably get feedback or requests for changes to your pull request.
This is a big part of the submission process, so don't be discouraged! Some contributors may sign off on the pull
request right away, others may have more detailed comments or feedback. This is a necessary part of the process in order
to evaluate whether the changes are correct and necessary.

**Any community member can comment on a PR, so you might get conflicting feedback**. Keep an eye out for comments from code
owners to provide guidance in such cases.

##### Be aware of the person behind the code

Be aware that _how_ you communicate requests and reviews in your feedback can have a significant impact on the success
of the pull request. Yes, we may merge a particular change that makes the project better, but the individual might just not
want to have anything to do with zkVMs-benchmarks ever again. The goal is not just having good code.

##### Abandoned or stale pull requests

If a pull request appears to be abandoned or stalled, it is polite to first check with the contributor to see if they
intend to continue the work before checking if they would mind if you took it over (especially if it just has nits
left). When doing so, it is courteous to give the original contributor credit for the work they started, either by
preserving their name and e-mail address in the commit log, or by using the `Author: ` or `Co-authored-by: ` metadata
tag in the commits.

_Adapted from the [Reth contributing guide](https://raw.githubusercontent.com/paradigmxyz/reth/main/CONTRIBUTING.md)_

[rust-coc]: https://github.com/rust-lang/rust/blob/master/CODE_OF_CONDUCT.md

[mcve]: https://stackoverflow.com/help/mcve
