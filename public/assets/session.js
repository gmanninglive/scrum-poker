function __swcpack_require__(mod) {
    function interop(obj) {
        if (obj && obj.__esModule) {
            return obj;
        } else {
            var newObj = {};
            if (obj != null) {
                for(var key in obj){
                    if (Object.prototype.hasOwnProperty.call(obj, key)) {
                        var desc = Object.defineProperty && Object.getOwnPropertyDescriptor ? Object.getOwnPropertyDescriptor(obj, key) : {};
                        if (desc.get || desc.set) {
                            Object.defineProperty(newObj, key, desc);
                        } else {
                            newObj[key] = obj[key];
                        }
                    }
                }
            }
            newObj.default = obj;
            return newObj;
        }
    }
    var cache;
    if (cache) {
        return cache;
    }
    var module = {
        exports: {}
    };
    mod(module, module.exports);
    cache = interop(module.exports);
    return cache;
}
var load = __swcpack_require__.bind(void 0, function(module, exports) {
    (function(global, factory) {
        typeof exports === "object" && typeof module !== "undefined" ? module.exports = factory() : typeof define === "function" && define.amd ? define(factory) : (global = typeof globalThis !== "undefined" ? globalThis : global || self, function() {
            var current = global.Cookies;
            var exports = global.Cookies = factory();
            exports.noConflict = function() {
                global.Cookies = current;
                return exports;
            };
        }());
    })(this, function() {
        "use strict";
        function assign(target) {
            for(var i = 1; i < arguments.length; i++){
                var source = arguments[i];
                for(var key in source)target[key] = source[key];
            }
            return target;
        }
        var defaultConverter = {
            read: function(value) {
                if (value[0] === '"') value = value.slice(1, -1);
                return value.replace(/(%[\dA-F]{2})+/gi, decodeURIComponent);
            },
            write: function(value) {
                return encodeURIComponent(value).replace(/%(2[346BF]|3[AC-F]|40|5[BDE]|60|7[BCD])/g, decodeURIComponent);
            }
        };
        function init(converter, defaultAttributes) {
            function set(name, value, attributes) {
                if (typeof document === "undefined") return;
                attributes = assign({}, defaultAttributes, attributes);
                if (typeof attributes.expires === "number") attributes.expires = new Date(Date.now() + attributes.expires * 864e5);
                if (attributes.expires) attributes.expires = attributes.expires.toUTCString();
                name = encodeURIComponent(name).replace(/%(2[346B]|5E|60|7C)/g, decodeURIComponent).replace(/[()]/g, escape);
                var stringifiedAttributes = "";
                for(var attributeName in attributes){
                    if (!attributes[attributeName]) continue;
                    stringifiedAttributes += "; " + attributeName;
                    if (attributes[attributeName] === true) continue;
                    stringifiedAttributes += "=" + attributes[attributeName].split(";")[0];
                }
                return document.cookie = name + "=" + converter.write(value, name) + stringifiedAttributes;
            }
            function get(name) {
                if (typeof document === "undefined" || arguments.length && !name) return;
                var cookies = document.cookie ? document.cookie.split("; ") : [];
                var jar = {};
                for(var i = 0; i < cookies.length; i++){
                    var parts = cookies[i].split("=");
                    var value = parts.slice(1).join("=");
                    try {
                        var found = decodeURIComponent(parts[0]);
                        jar[found] = converter.read(value, found);
                        if (name === found) break;
                    } catch (e) {}
                }
                return name ? jar[name] : jar;
            }
            return Object.create({
                set,
                get,
                remove: function(name, attributes) {
                    set(name, "", assign({}, attributes, {
                        expires: -1
                    }));
                },
                withAttributes: function(attributes) {
                    return init(this.converter, assign({}, this.attributes, attributes));
                },
                withConverter: function(converter) {
                    return init(assign({}, this.converter, converter), this.attributes);
                }
            }, {
                attributes: {
                    value: Object.freeze(defaultAttributes)
                },
                converter: {
                    value: Object.freeze(converter)
                }
            });
        }
        var api = init(defaultConverter, {
            path: "/"
        });
        return api;
    });
});
var { default: Cookie  } = load();
var user_list = [];
var votes = {};
var id = window.location.pathname.replace("/session/", "");
var edit_user_form = document.getElementById("edit_user_form");
var user_list_container = document.getElementById("user_list_container");
var user_list_template = document.getElementById("user_node_template");
var username = get_username();
function get_username() {
    return Cookie.get("sp_user");
}
if (edit_user_form) {
    edit_user_form.addEventListener("submit", function(e) {
        e.preventDefault();
        var target = e.target;
        var username = target.display_name.value;
        Cookie.set("sp_user", username, {
            sameSite: "strict"
        });
        edit_user_form.classList.add("hidden");
        init_ws(username);
    });
    if (username) init_ws(username);
    else edit_user_form.classList.toggle("hidden");
}
function init_ws(username) {
    var socket = new WebSocket("ws://localhost:3000/ws/".concat(id));
    socket.addEventListener("open", function(event) {
        socket.send(username);
    });
    socket.addEventListener("message", function(event) {
        console.log("Message from server ", event.data);
        var data = JSON.parse(event.data);
        if ("Vote" in data.payload) {
            var Vote = data.payload.Vote;
            votes[Vote.user] = Vote.vote;
        }
        user_list = data.users || [];
        render_user_list();
    });
    document.querySelectorAll("[id^='poker_card_']").forEach(function(card) {
        card.addEventListener("click", function(e) {
            e.preventDefault();
            var message = {
                Vote: {
                    user: username,
                    vote: +card.id.replace("poker_card_", "")
                }
            };
            socket.send(JSON.stringify(message));
        });
    });
}
function render_user_list() {
    if (user_list_container && user_list_template) {
        user_list_container.replaceChildren();
        user_list.map(function(user) {
            var template = user_list_template.cloneNode(true).content.firstElementChild;
            template.firstElementChild.innerText = user;
            var vote_display = template.lastElementChild.firstElementChild;
            var vote = votes[user];
            vote_display.innerText = vote ? "Voted ".concat(vote) : "Voting...";
            user_list_container.appendChild(template);
        });
    }
}