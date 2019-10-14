Vue.component("connect-form", {
	data: function () {
		return {
			nick: "",
		};
	},

	mounted() {
		let nick = sessionStorage.getItem("nick");
		if (nick !== null) {
			this.nick = nick;
		}
	},

	template: `
<div>
	<label for="nick">Nick: </label><input type="text" name="nick" v-model="nick">
	<button v-on:click="$emit('connect', nick)">Connect</button>
</div>`,
});

const MsgType = {
	Connected: "Connected",
	Ping: "Ping",
	Message: "Message",
	YourNickIsTaken: "YourNickIsTaken",
};

window.onload = () => (new Vue({
	el: "#app",

	data: {
		eventSource: null,
		user_msg: "",
		nick: "",
		messages: [],
		connected: false,
	},

	mounted: function () {
	},

	methods: {
		handle_connect: function (nick) {
			this.nick = nick;
			console.log(`connecting as ${this.nick}`);
			this.connect();
		},
		connect: function () {
			this.eventSource = new EventSource(encodeURI(`/events?nick=${this.nick}`));

			this.eventSource.onopen = event => console.log(event);
			this.eventSource.onerror = event => console.log(event);
			this.eventSource.onmessage = this.handle_message;
		},
		handle_message: function(event) {
			console.log(event.data);
			let msg = JSON.parse(event.data);

			switch (msg.type) {
				case MsgType.Connected:
					console.log("Connected");
					sessionStorage.setItem("nick", this.nick);
					this.connected = true;
					break;
				case MsgType.Ping: break;
				case MsgType.YourNickIsTaken:
					alert("Your nick is taken");
					break;
				case MsgType.Message:
					this.messages.push(msg.data);
					break;
				default:
					console.log("Unknown type");
					break;
			}
		},
		send_msg: function() {
			let xhr = new XMLHttpRequest();
			xhr.open("POST", "/send_msg", true);
			xhr.onload = () => {
				if (xhr.status !== 200) {
					console.log("request failed");
					return;
				}
				this.user_msg = "";
			};
			xhr.setRequestHeader("content-type", "application/json");
			xhr.send(JSON.stringify(this.user_msg));
		},
	},
}));