# current todo:

-   limit player input length

-   clients may or may not be correctly reporting disconnections, look into this

-   clean up frontend state

-   send current countdown to new clients joining, also input, maybe previous winner?

-   use unwrap less

-   room owner able to start game early

-   probably look into using dashmap instead of std hashmap for global game data for concurrent references

-   probably use parking_lot because panics with a mutex lock DONT poison i think

-   future word games:
    -   anagrams
    -   greentea
