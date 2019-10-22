Vue.component("pastes", {
	props: ["nick", "pastes"],

	data: function () {
		return {
			title_input: "",
			content_input: "",
		};
	},

	methods: {
		upload: function(_event) {
			let xhr = new XMLHttpRequest();
			xhr.open("POST", "/send_paste", true);
			xhr.onload = () => {
				if (xhr.status !== 200) {
					console.log("request failed");
					return;
				}
				this.title_input = "";
				this.content_input = "";
			};
			xhr.setRequestHeader("content-type", "application/json");
			xhr.send(JSON.stringify({
				title: this.title_input,
				content: this.content_input,
			}));
		}
	},

	template: `
<div>
	<input type="text" v-model="title_input">
	<textarea 
		name="content"
		id="content_input"
		placeholder="Paste content here..."
		v-model="content_input"
		required></textarea>
	<input type="button" value="Upload" v-on:click="upload">
	<div v-for="paste in pastes">
		{{ paste.author }}: {{ paste.title }}
		<pre>{{ paste.content }}</pre>
	</div>
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
	Paste: "Paste",
};

Vue.component("chat", {
	props: ["nick", "messages"],

	data: function () {
		return {
			user_msg: "",
		};
	},

	activated: function () {
		this.scroll_to_bottom();
	},

	watch: {
		messages: function (new_val, old_val) {
			this.scroll_to_bottom();
		}
	},

	methods: {
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

		eventSource: null,
		connected: false,
		has_unread_msg: false,

		nick: "",
		messages: [],
		pastes: [],
	},

	computed: {
		tab_props: function () {
			switch (this.current_tab) {
				case "chat":
					return {
						nick: this.nick,
						messages: this.messages,
					};
				case "pastes":
					return {
						nick: this.nick,
						pastes: this.pastes,
					};
			}
		},
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

					this.messages.push(...msg.data[0]);
					this.pastes.push(...msg.data[1]);

					localStorage.setItem("nick", this.nick);
					this.connected = true;

					break;
				case MsgType.Ping:
					break;
				case MsgType.Message:
					this.messages.push(msg.data);
					this.notify();

					break;
				case MsgType.Paste:
					this.pastes.push(msg.data);
					this.notify();

					break;
				default:
					console.log("Unknown type");
					break;
			}
		},
		notify: function () {
			if (!document.hasFocus() && !this.has_unread_msg) {
				this.has_unread_msg = true;
				document.title = "* " + document.title;
			}
		},
		on_focus: function () {
			if (this.has_unread_msg) {
				this.has_unread_msg = false;
				document.title = document.title.substring(2);
			}
		},
		mounted: function () {
			window.addEventListener("focus", this.on_focus);
		},
	},
}));
