const socket = new WebSocket("/spectate");
const boardDiv = document.getElementById("board");
const leaderboardDiv = document.getElementById("leaderboard");

socket.addEventListener("open", (event) => {
    console.log("Connected");
});

let colorCache = {};
function getRandomColorForPlayerId(playerId) {
    if (!(playerId in colorCache)) {
        // Function to generate a random number between 0 and 255
        function getRandomInt() {
            return Math.floor(Math.random() * 256);
        }

        // Convert a number to a two-digit hexadecimal string
        function toHex(number) {
            return number.toString(16).padStart(2, "0");
        }

        let red, green, blue;

        // Keep generating colors until we get one that is bright enough
        do {
            red = getRandomInt();
            green = getRandomInt();
            blue = getRandomInt();
        } while (red + green + blue <= 400);

        // Convert each value to a hexadecimal string and concatenate
        const rgbHex = `#${toHex(red)}${toHex(green)}${toHex(blue)}`;

        colorCache[playerId] = rgbHex;
    }
    return colorCache[playerId];
}

function renderState() {
    const spaces = gameState["spaces"]

    boardDiv.innerHTML = "";

    const table = document.createElement("table");
    table.classList.add("game-board");

    let playerStats = {};

    for (const col of spaces) {
        const tr = document.createElement("tr");
        for (const cell of col) {
            if (cell["owner"] !== undefined) {
                if (playerStats[cell["owner"]] === undefined) {
                    playerStats[cell["owner"]] = { land: 0, units: 0 };
                }
                playerStats[cell["owner"]].land++;
                playerStats[cell["owner"]].units += cell["units"];
            }
            const td = document.createElement("td");
            if (cell["type"] == "PlayerCapital") {
                td.innerHTML = `P<br />${cell["units"]}`;
                td.style.backgroundColor = getRandomColorForPlayerId(
                    cell["owner"],
                );
            } else if (cell["type"] == "PlayerTown") {
                td.innerHTML = `p<br />${cell["units"]}`;
                td.style.backgroundColor = getRandomColorForPlayerId(
                    cell["owner"],
                );
            } else if (cell["type"] == "NeutralTown") {
                td.innerHTML = `t<br />${cell["units"]}`;
                td.classList.add(`neutralTown`);
            } else if (cell["type"] == "PlayerEmpty") {
                td.innerHTML = `${cell["units"]}`;
                td.style.backgroundColor = getRandomColorForPlayerId(
                    cell["owner"],
                );
            } else if (cell["type"] == "Empty") {
                td.innerHTML = "";
            } else if (cell["type"] == "Mountain") {
                td.innerHTML = "M";
                td.classList.add(`mountain`);
            } else {
                alert("Bad space type");
            }
            td.classList.add("space");
            table.appendChild(td);
        }
        table.appendChild(tr);
    }
    boardDiv.replaceChildren(table);

    const leaderboard = document.createElement("table");
    for (const [key, value] of Object.entries(playerStats).sort((a, b) =>
        a[0].localeCompare(b[0]),
    )) {
        const tr = document.createElement("tr");
        tr.style.backgroundColor = getRandomColorForPlayerId(key);

        const td1 = document.createElement("td");
        td1.innerText = `Player ${key}`;
        tr.appendChild(td1);

        const td2 = document.createElement("td");
        td2.innerText = `Land: ${value.land}`;
        tr.appendChild(td2);

        const td3 = document.createElement("td");
        td3.innerText = `Units: ${value.units}`;
        tr.appendChild(td3);

        leaderboard.appendChild(tr);
    }
    leaderboardDiv.replaceChildren(leaderboard);
}

function collapseDeltas(deltas) {
    let deltaMap = new Map();
    deltas.forEach(delta => delta.forEach(change => {
        let key = change["coord"].x + "," + change["coord"].y;
        if (deltaMap.has(key)) {
            deltaMap.get(key)["next"] = change["next"];
        } else {
            deltaMap.set(key, { ...change });
        }
    }));
    return [...deltaMap.values()];
}

function goToTurn(turn) {
    if (turn < 0 || turn > deltas.length || gameState["turn"] == turn) {
        return;
    }

    if (gameState["turn"] > turn) {
        let collapsedDelta = collapseDeltas(deltas.slice(turn, gameState["turn"]));
        applyDeltaBackward(collapsedDelta);
    } else {
        let collapsedDelta = collapseDeltas(deltas.slice(gameState["turn"], turn));
        applyDeltaForward(collapsedDelta);
    }

    document.getElementById("turn-input").value = turn;
    gameState["turn"] = turn;
}

function applyDeltaForward(delta) {
    delta.forEach(change => {
        coord = change["coord"];
        gameState["spaces"][coord.x][coord.y] = change["next"];
    });
}

function applyDeltaBackward(delta) {
    delta.forEach(change => {
        coord = change["coord"];
        gameState["spaces"][coord.x][coord.y] = change["prev"];
    });
}

function onClickForward() {
    viewLatest = false;
    goToTurn(gameState["turn"] + 1);
    renderState();
}

function onClickBackward() {
    viewLatest = false;
    goToTurn(gameState["turn"] - 1);
    renderState();
}

function onInputChange(turnString) {
    viewLatest = false;
    goToTurn(parseInt(turnString));
    renderState();
}

function onClickLatest() {
    viewLatest = true;
    goToTurn(deltas.length);
    renderState();
}

let deltas = [];
let gameState = { "game_id": undefined, "spaces": undefined, "turn": 0, }
let viewLatest = true;

// Listen for messages
socket.addEventListener("message", (event) => {
    const data = JSON.parse(event.data);
    if (data["initial_game_state"] !== undefined) {
        gameState = data["initial_game_state"];
    }
    if (data["deltas"] !== undefined) {
        deltas = deltas.concat(data["deltas"]);
        document.getElementById("turn-input").setAttribute("max", deltas.length);
        document.getElementById("total-turns").innerText = "/ " + deltas.length;
        if (viewLatest) {
            goToTurn(deltas.length);
            renderState();
        }
    }
});
