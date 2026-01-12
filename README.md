# ff-db a simple, flat-file database for all your simple data storage needs
## Features
 A sql engine; currently supports: 'WHERE', 'SELECT', 'INSERT', 'AND' keywords, \
    left-to-right arithemtic, as well as'!=', '=', '<', '' inside of where clauses \
 A csv-like database format with column typing and names 

## Planned features
 expanding the sql engine, especially support for the 'OR' and 'LIKE' keywords.\
 UX improvments    \
    - A integration guide \
    - A scaffolding script \
    - Completing the error handling \
    - Public API for the DB object if tighter integration is desired \
    - A binary communication endpoint 

## Planned improvments/changes
 A slight refactor in the Engine object \
 Support for mixing 'OR' with 'AND' propably with left-to-right precedence

## Uses for this
 Tiny projects, examples include: \
    - My use for this which is simple a workshop inventory tool \
    - Internal/code-adjecent tooling \
 Bigger projects that make use of a database as side functionality and will benenfit from the simplicity 

## Not uses for this
 Anything that uses the DB often
