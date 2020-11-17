# VCX Agent Core
VCX Agent Core is implementation of simple agent with persistent file storage. It's based on
[NodeJS Wrapper](../../../wrappers/node) for `aries-vcx` library.

# Try it
1. First step is to [compile](../../../libvcx) `aries-vcx` library and make it available on your system.
2. You need to have `aries-vcx` compatible agency ready. See more [info](../../../README.md).
3. Run `test:threaded:integration`. This will run sample scenario where:Alice and Faber
    - establish Aries connection, 
    - Faber issues credential to Alice
    - Faber requests Alice to prove certain information using the credential. 
    
# Note 
You can also have look at [vcxagent-cli](../vcxagent-cli) - simple CLI Aries agent based 
on this project.