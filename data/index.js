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

    for (const row of spaces) {
        const tr = document.createElement("tr");
        for (const col of row) {
            const td = document.createElement("td");
            if (col["type"] == "PlayerCapital") {
                td.innerHTML = `P<br />${col["units"]}`;
                td.classList.add(`player${col["owner"]}`);
            } else if (col["type"] == "PlayerTown") {
                td.innerHTML = `p<br />${col["units"]}`;
                td.classList.add(`player${col["owner"]}`);
            } else if (col["type"] == "NeutralTown") {
                td.innerHTML = `t<br />${col["units"]}`;
                td.classList.add(`neutralTown`);
            } else if (col["type"] == "PlayerEmptySpace") {
                td.innerHTML = `${col["units"]}`;
                td.classList.add(`player${col["owner"]}`);
            } else if (col["type"] == "EmptySpace") {
                td.innerHTML = "";
            } else if (col["type"] == "Mountain") {
                td.innerHTML = "M";
                td.classList.add(`mountain`);
            }
            td.classList.add("space");
            table.appendChild(td);
        }
        table.appendChild(tr);
    }

    contentDiv.replaceChildren(table);
});
