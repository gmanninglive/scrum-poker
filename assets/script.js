import Cookie from "https://cdn.jsdelivr.net/npm/js-cookie@3.0.5/+esm";

let user_list = [];
let votes = {};

const id = window.location.pathname.replace("/session/", "");
const edit_user_form = document.getElementById("edit_user_form");
const user_list_container = document.getElementById("user_list_container");
const user_list_template = document.getElementById("user_node_template");

function get_username() {
  return Cookie.get("sp_user");
}

edit_user_form.addEventListener("submit", (e) => {
  e.preventDefault();
  const username = e.target.display_name.value;

  Cookie.set("sp_user", username, { sameSite: "strict" });

  edit_user_form.classList.add("hidden");

  init_ws(username);
});

if (Cookie.get("sp_user")) {
  init_ws(get_username());
} else {
  edit_user_form.classList.toggle("hidden");
}

function init_ws(username) {
  const socket = new WebSocket(`ws://localhost:3000/ws/${id}`);

  socket.addEventListener("open", function (event) {
    socket.send(username);
  });

  socket.addEventListener("message", function (event) {
    console.log("Message from server ", event.data);

    const data = JSON.parse(event.data);
    switch (data.type) {
      case "user_voted": {
        const { payload } = data;
        votes[payload.username] = payload.vote;
        break;
      }
      default: {
      }
    }

    user_list = data.users || [];
    render_user_list();
  });

  document.querySelectorAll("[id^='poker_card_']").forEach((card) => {
    card.addEventListener("click", (e) => {
      e.preventDefault();
      socket.send(card.id.replace("poker_card_", ""));
    });
  });
}

function render_user_list() {
  user_list_container.replaceChildren();
  user_list.map((user) => {
    const template =
      user_list_template.cloneNode(true).content.firstElementChild;

    template.firstElementChild.innerText = user;

    const vote_display = template.lastElementChild.firstElementChild;
    const vote = votes[user];
    vote_display.innerText = vote ? `Voted ${vote}` : "Voting...";

    user_list_container.appendChild(template);
  });
}
