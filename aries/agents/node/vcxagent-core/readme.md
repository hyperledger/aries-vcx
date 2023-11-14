# VCX Agent Core
VCX Agent Core is implementation of simple agent with persistent file storage. It's based on
[NodeJS Wrapper](../../../wrappers/node) for `aries-vcx` library.

# Try it
1. First step is to compile and `aries-vcx` and make it available on your system. Follow [instructions](../../../libvcx).
2. You need to have mediator agent compatible with `aries-vcx`. See more [info](../../../README.md).
3. Run `npm run demo`. This will run sample scenario where Alice and Faber
    - establish connection, 
    - Faber issues a credential to Alice
    - Faber requests Alice to prove certain information about herself (using the credential). 
    
# Note 
You can also have look at [vcxagent-cli](../vcxagent-cli) - CLI Aries agent based 
on this project.
