# current todo:

-   clients may or may not be correctly reporting disconnections, look into this

-   send previous winner to new clients joining?

-   dont use `.allow_origin(cors::Any)`

-   store previous winners

-   possibly care about duplicate usernames

-   probably look into using dashmap instead of std hashmap for global game data for concurrent references

-   probably use parking_lot because panics with a mutex lock DONT poison i think

-   future word games:
    -   anagrams
    -   greentea
