const socket = new WebSocket("/spectate");
const contentDiv = document.getElementById("content");

socket.addEventListener("open", (event) => {
    console.log("Connected");
});

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
                td.classList.add(`player${cell["owner"]}`);
            } else if (cell["type"] == "PlayerTown") {
                td.innerHTML = `p<br />${cell["units"]}`;
                td.classList.add(`player${cell["owner"]}`);
            } else if (cell["type"] == "NeutralTown") {
                td.innerHTML = `t<br />${cell["units"]}`;
                td.classList.add(`neutralTown`);
            } else if (cell["type"] == "PlayerEmpty") {
                td.innerHTML = `${cell["units"]}`;
                td.classList.add(`player${cell["owner"]}`);
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
    for (const [key, value] of Object.entries(playerStats)) {
        const tr = document.createElement("tr");
        tr.classList.add(`player${key}`);

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
