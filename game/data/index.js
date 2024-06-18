const socket = new WebSocket("/spectate");
const contentDiv = document.getElementById("content");

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

// Listen for messages
socket.addEventListener("message", (event) => {
    const spaces = JSON.parse(event.data)["spaces"];

    contentDiv.innerHTML = "";

    const table = document.createElement("table");

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

    contentDiv.replaceChildren(table, leaderboard);
});
