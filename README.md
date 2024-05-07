# How to run

-   First, start your AI(s) on some non-8080 port(s).
-   Then, `cd game` and `cargo run -- <LIST OF PORTS>`. For example `cargo run -- 8081 8082 8083 8084 8084 8084` to start a game with 6 players, 3 of which are using the same AI
-   You can use the sample AIs in the `ai` folder with `cargo run -p random-ai -- <PORT TO RUN ON>` or `cargo run -p jroylance-ai -- <PORT TO RUN ON>`
-   You can spectate the running game by visiting `localhost:8080` in a web browser.

# Missing features

-   The game doesn't end when there's only one player left. There's also no concept of an AI being out of the game, they are just unable to make any move
