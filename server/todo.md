# current todo:

-   stop using anyhow

-   actual auth error messages

-   delete expired sessions on app start

-   client and server version checking

-   review all `.unwrap()`s

-   use tracing

-   store and display previous winners of session

-   dont use `.allow_origin(cors::Any)`

-   probably look into using dashmap instead of std hashmap for global game data for concurrent references

-   probably use parking_lot because panics with a mutex lock DONT poison i think

-   future word games:
    -   greentea
