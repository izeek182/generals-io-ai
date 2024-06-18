# How to run

-   First, start your AI(s) on some arbitrary port(s).
-   Then, `cargo run -p game -- <LIST OF AI PORTS>`. For example `cargo run -p game -- 8081 8082 8083 8084 8084 8084` to start a game with 6 players, 3 of which are using the same AI
-   You can use the sample AIs in the `ai` folder with `cargo run -p random-ai -- <PORT TO RUN ON>` or `cargo run -p jroylance-ai -- <PORT TO RUN ON>`
-   You can spectate the running game by visiting the url that `cargo run -p game` outputs in a web browser.

# Architecture

-   Every turn, the game server will make an http request to each of the list of ports passed in. It will send the game state as a json blob, and expects a valid move in response.

# TODOs

-   Leave 1 unit behind when making a move
-   Artwork on spaces
-   Associate ports/ais with player ids somehow
