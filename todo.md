important:

- client and server version checking
- custom avatars
- server db doesn't get saved between container updates (use docker volume)
- word bomb fastest guess may be very wrong sometimes

qol:

- make chat resizable
- remove some letters from word bomb extra life thing by default (x, z)
- back button after create/join game in homepage
- store and display previous winners

future:

- singleplayer
- future word games:
  - greentea
- difficulty settings in anagrams, more customisable word bomb settings
- sound effects (never happening)
- gamemodes with powerups...?

doesn't really matter:

- better page transitions (solid-transition-group)
- more leniency on usernames possibly containing profanity
- send notification to new party leader

code qual:

- use tracing
- review all `.unwrap()`s
