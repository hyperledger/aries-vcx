# Roadmap 2023

### Modularization
This is one of the main themes for 2023. We'll start by publishing smaller subcrates on crates.io, such as crates encompassing aries messages, ddo resolver or diddoc. We hope this would attract more users and contributors who might be interested in smaller parts of the codebase if not the entire aries-vcx crate.

### Credex libs support
We are currently at point where credex libs can be used with holder, prover and verifier, but missing support for issuer on top of credex. Once that is done, we'll continue with effort to add support for Askar wallet.

### Ledger agnosticity
Following up the spin off Hyperldger Anoncreds as separate project, we expect issuer's anchoring their DIDs, CredDefs etc. on ledgers other than Indy. We'll be followng developments in this area and start enabling support for non-indy credentials, although this is arguably much wider community effort. The first action step forward will be implementing DDO resolver interface and support for an additional method other than did:sov

### Community engagement
In second half of 2022 we've started to put much more focus on community - increased discord presence, decreased time to review PRs, started weekly community calls - it didn't take long till we could observe increase in contributors and discord activity. We would like to  maintaining this culture and activities, but also further become more inviting to new contributors by improving documentation, lowering barriers starting an aries-vcx project off the ground.

### And more
widening aries protocol support (didexchange, newer version of issuance and presentation protocols); increasing AATH coverage, enhancing code quality, testing speed, coverege; exploring didcomm 2.0; starting of new projects on top of aries-vcx (pickup protocol compliant mediator, cli tools) - are also on the list.
