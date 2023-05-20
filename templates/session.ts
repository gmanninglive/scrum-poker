import Cookie from "js-cookie";
import { WSResponse } from "../bindings/WSResponse";
import { UserMessage } from "../bindings/UserMessage";

let user_list: string[] = [];
let votes: Record<string, number> = {};

const id = window.location.pathname.replace("/session/", "");
const edit_user_form = document.getElementById("edit_user_form");
const user_list_container = document.getElementById("user_list_container");
const user_list_template = document.getElementById("user_node_template");
let username = get_username();

function get_username() {
  return Cookie.get("sp_user");
}

if (edit_user_form) {
  edit_user_form.addEventListener("submit", (e: any) => {
    e.preventDefault();

    const target = e.target! as { display_name: { value: string } };
    const username = target.display_name.value as string;

    Cookie.set("sp_user", username, { sameSite: "strict" });

    edit_user_form.classList.add("hidden");

    init_ws(username);
  });

  if (username) {
    init_ws(username);
  } else {
    edit_user_form.classList.toggle("hidden");
  }
}

function init_ws(username: string) {
  let scheme = window.location.protocol.startsWith("https")
    ? "wss://"
    : "ws://";

  const socket = new WebSocket(`${scheme}${window.location.host}/ws/${id}`);

  socket.addEventListener("open", function (event) {
    socket.send(username);
  });

  socket.addEventListener("message", function (event) {
    console.log("Message from server ", event.data);

    const data: WSResponse = JSON.parse(event.data);

    if ("Vote" in data.payload) {
      const {
        payload: { Vote },
      } = data;
      votes[Vote.user] = Vote.vote;
    }

    user_list = data.users || [];
    render_user_list();
  });

  document.querySelectorAll("[id^='poker_card_']").forEach((card) => {
    card.addEventListener("click", (e) => {
      e.preventDefault();
      const message: UserMessage = {
        Vote: {
          user: username,
          vote: +card.id.replace("poker_card_", ""),
        },
      };

      socket.send(JSON.stringify(message));
    });
  });
}

function render_user_list() {
  if (user_list_container && user_list_template) {
    user_list_container.replaceChildren();
    user_list.map((user) => {
      const template =
        //@ts-ignore
        user_list_template.cloneNode(true).content.firstElementChild;

      template.firstElementChild.innerText = user;

      const vote_display = template.lastElementChild.firstElementChild;
      const vote = votes[user];
      vote_display.innerText = vote ? `Voted ${vote}` : "Voting...";

      user_list_container.appendChild(template);
    });
  }
}
