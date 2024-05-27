# current todo:

-   use tracing

-   add username length limit

-   store and display previous winners of session

-   dont use `.allow_origin(cors::Any)`

-   save username in localstorage

-   probably look into using dashmap instead of std hashmap for global game data for concurrent references

-   probably use parking_lot because panics with a mutex lock DONT poison i think

-   future word games:
    -   greentea
