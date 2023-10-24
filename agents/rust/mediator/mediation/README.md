> 🚧🚧**WIP**🚧🚧: Integration into aries-vcx in process. See main branch for the MVP http service.

# Aries mediator service (http)

## History

This repo was intended to be a test project to try Rust and Axum features.

Over time it has turned into a ~fully functional aries mediator service supporting the following Aries concepts over ***http*** endpoints.

> **Caveat**: Authentication must be handled at another layer. Integration with [aries-vcx] is intended to enable this.

- [Mediator coordination protocol](https://github.com/hyperledger/aries-rfcs/blob/main/features/0211-route-coordination/README.md)
- [Core:Routing: Forward Message](https://github.com/hyperledger/aries-rfcs/blob/main/concepts/0094-cross-domain-messaging/README.md#corerouting10forward)
- [Pickup protocol 2.0](https://github.com/hyperledger/aries-rfcs/tree/main/features/0685-pickup-v2)

[aries-vcx]: https://github.com/hyperledger/aries-vcx

Original readme is available below.

> # axum-test-server
>
> A axum server app to test and learn Rust and Axum features.
>
> The commits are made in such a way as to demonstrate and try one feature or workflow (refactoring) at a time.  
> So the git log could be educative.  
>
> ## Some Interesting commits
>
> 1. [feat(main.rs): :thread: Demo server as in axum docs](https://github.com/nain-F49FF806/axum-test-server/commit/d7fceaf9b731251cdbe8642c716dfaa3a697349a)
>
> 2. [refactor: `use` too bring names into local namespace](https://github.com/nain-F49FF806/axum-test-server/commit/f3a58597fc05fe5353140e764d532446ff10000e)
>
> 3. [refactor: break off some code into module and load module in main](https://github.com/nain-F49FF806/axum-test-server/commit/253956dc30866f516f484c6ef549c55054cb9f3f)
>
> 4. [refactor: Use functions in sibling module by declaring their path from root crate](https://github.com/nain-F49FF806/axum-test-server/commit/f7a5020ba52876e463c86efb3390af527e09990c#r121244243)
>
> 5. [refactor: modules can have sub-modules, which can then be loaded by others using path from root crate (*if made public*)](https://github.com/nain-F49FF806/axum-test-server/commit/877cf3bac05d9cf786db3ae45202b2d4d9a98a5c)
>
> 6. [feat(json): serde macros help with serializing, deserializing from Rust structs to JSON wire format](https://github.com/nain-F49FF806/axum-test-server/commit/505ec1ec8fc6169620be235231643f678bab20ff)
>
>
> ## Notess
>
> You can also look at a blog post with some personal notes (mostly for the Rust side) here : [running-notes-on-rust-and-axum-framework]
>
> [running-notes-on-rust-and-axum-framework]: https://envs.net/~nain/aries-vcx-diaries/running-notes-on-rust-and-axum-framework-ft-tutorial-course-by-brooks-builds.html
>