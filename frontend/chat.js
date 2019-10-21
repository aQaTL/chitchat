Vue.component("pastes", {
	template: `
<div>
	<h1>Hello, World!</h1>
</div>
	`,
});

Vue.component("connect-form", {
	data: function () {
		return {
			nick: "",
		};
	},

	mounted() {
		let nick = localStorage.getItem("nick");
		if (nick !== null) {
			this.nick = nick;
			this.$emit("connect", this.nick);
		}
	},

	template: `
<div class="nick_input">
	<input 
		type="text"
		name="nick"
		id="nick"
		placeholder="Nick"
		v-model="nick"
		v-on:keyup.enter="$emit('connect', nick)">
</div>`,
});

const MsgType = {
	Connected: "Connected",
	Ping: "Ping",
	Message: "Message",
	YourNickIsTaken: "YourNickIsTaken",
};

Vue.component("chat", {
	data: function () {
		return {
			eventSource: null,
			user_msg: "",
			nick: "",
			messages: [],

			connected: false,
			has_unread_msg: false,
		};
	},

	mounted: function () {
		window.addEventListener("focus", this.on_focus);
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
		handle_message: function (event) {
			console.log(event.data);
			let msg = JSON.parse(event.data);

			switch (msg.type) {
				case MsgType.Connected:
					console.log("Connected");

					this.messages.push(...msg.data);
					this.scroll_to_bottom();

					localStorage.setItem("nick", this.nick);
					this.connected = true;

					break;
				case MsgType.Ping:
					break;
				case MsgType.YourNickIsTaken:
					console.log("Your nick is taken");
					break;
				case MsgType.Message:
					this.messages.push(msg.data);
					this.scroll_to_bottom();

					if (!document.hasFocus() && !this.has_unread_msg) {
						this.has_unread_msg = true;
						document.title = "* " + document.title;
					}
					break;
				default:
					console.log("Unknown type");
					break;
			}
		},
		send_msg: function () {
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
		on_focus: function () {
			if (this.has_unread_msg) {
				this.has_unread_msg = false;
				document.title = document.title.substring(2);
			}
		},
		scroll_to_bottom: function () {
			Vue.nextTick(() => {
				let msgs = this.$refs.messages;
				msgs.scrollTop = msgs.scrollHeight;
			});
		},
	},

	filters: {
		time: (date_str) => new Date(date_str).toLocaleTimeString(),
	},

	template: `
<div>
	<connect-form id="connect_form" v-if="!connected" v-on:connect="handle_connect"></connect-form>

	<section id="messages" ref="messages">
		<div>
			<div class="message" v-for="msg in messages">
				<span>[{{ msg.time | time }}] </span><span>{{ msg.nick }}</span>: <span>{{ msg.msg }}</span>
			</div>
		</div>
	</section>
	<section id="input">
		<label for="msg_input">{{ nick }}</label>
		<input type="text" v-model="user_msg" id="msg_input" v-on:keyup.enter="send_msg()">
	</section>
</div>
`,
});

window.onload = () => (new Vue({
	el: "#app",

	data: {
		current_tab: "chat",
		tabs: ["chat", "pastes"],
	},
}));
