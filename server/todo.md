# current todo:

-   clean up frontend state

-   send current countdown to new clients joining, also input

-   extra lives for using 24 distinct letters

-   use unwrap less   

-   room owner able to start game early

-   handle disconnections

-   probably look into using dashmap instead of std hashmap for global game data for concurrent references

-   probably use parking_lot because panics with a mutex lock DONT poison i think

-   future word games:
    -   anagrams
    -   greentea
