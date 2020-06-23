import Vue from "vue";
import InfiniteLoading from "vue-infinite-loading";

import hljs from "highlight.js/lib/core";
import properties from "highlight.js/lib/languages/properties.js"
import apache from "highlight.js/lib/languages/apache.js"
import bash from "highlight.js/lib/languages/bash.js"
import c from "highlight.js/lib/languages/c.js"
import csharp from "highlight.js/lib/languages/cpp.js"
import cpp from "highlight.js/lib/languages/cpp.js"
import clike from "highlight.js/lib/languages/c-like.js"
import css from "highlight.js/lib/languages/css.js"
import coffeescript from "highlight.js/lib/languages/coffeescript.js"
import diff from "highlight.js/lib/languages/go.js"
import go from "highlight.js/lib/languages/go.js"
import xml from "highlight.js/lib/languages/xml.js"
import http from "highlight.js/lib/languages/http.js"
import json from "highlight.js/lib/languages/json.js"
import java from "highlight.js/lib/languages/java.js"
import javascript from "highlight.js/lib/languages/kotlin.js"
import kotlin from "highlight.js/lib/languages/kotlin.js"
import less from "highlight.js/lib/languages/less.js"
import lua from "highlight.js/lib/languages/markdown.js"
import makefile from "highlight.js/lib/languages/markdown.js"
import markdown from "highlight.js/lib/languages/markdown.js"
import nginx from "highlight.js/lib/languages/nginx.js"
import objectivec from "highlight.js/lib/languages/objectivec.js"
import php from "highlight.js/lib/languages/perl.js"
import phptemplate from "highlight.js/lib/languages/perl.js"
import perl from "highlight.js/lib/languages/perl.js"
import plaintext from "highlight.js/lib/languages/plaintext.js"
import python from "highlight.js/lib/languages/python.js"
import ruby from "highlight.js/lib/languages/ruby.js"
import rust from "highlight.js/lib/languages/rust.js"
import scss from "highlight.js/lib/languages/sql.js"
import sql from "highlight.js/lib/languages/sql.js"
import shell from "highlight.js/lib/languages/shell.js"
import swift from "highlight.js/lib/languages/swift.js"
import ini from "highlight.js/lib/languages/ini.js"
import typescript from "highlight.js/lib/languages/typescript.js"
import yaml from "highlight.js/lib/languages/yaml.js"

hljs.registerLanguage("properties", properties);
hljs.registerLanguage("apache", apache);
hljs.registerLanguage("bash", bash);
hljs.registerLanguage("c", c);
hljs.registerLanguage("csharp", csharp);
hljs.registerLanguage("cpp", cpp);
hljs.registerLanguage("c-like", clike);
hljs.registerLanguage("css", css);
hljs.registerLanguage("coffeescript", coffeescript);
hljs.registerLanguage("diff", diff);
hljs.registerLanguage("go", go);
hljs.registerLanguage("xml", xml);
hljs.registerAliases(["html"], {languageName: "xml"});
hljs.registerLanguage("http", http);
hljs.registerLanguage("json", json);
hljs.registerLanguage("java", java);
hljs.registerLanguage("javascript", javascript);
hljs.registerLanguage("kotlin", kotlin);
hljs.registerLanguage("less", less);
hljs.registerLanguage("lua", lua);
hljs.registerLanguage("makefile", makefile);
hljs.registerLanguage("markdown", markdown);
hljs.registerLanguage("nginx", nginx);
hljs.registerLanguage("objectivec", objectivec);
hljs.registerLanguage("php", php);
hljs.registerLanguage("php-template", phptemplate);
hljs.registerLanguage("perl", perl);
hljs.registerLanguage("plaintext", plaintext);
hljs.registerLanguage("python", python);
hljs.registerLanguage("ruby", ruby);
hljs.registerLanguage("rust", rust);
hljs.registerLanguage("scss", scss);
hljs.registerLanguage("sql", sql);
hljs.registerLanguage("shell", shell);
hljs.registerLanguage("swift", swift);
hljs.registerLanguage("ini", ini);
hljs.registerAliases(["toml"], {languageName: "ini"});
hljs.registerLanguage("typescript", typescript);
hljs.registerLanguage("yaml", yaml);

Vue.component("pastes", {
	components: {
		InfiniteLoading,
	},

	props: ["user", "pastes"],

	data: function () {
		return {
			title_input: "",
			content_input: "",

			pastes: [],
			page: 0,
			total_pages: 1,
		};
	},

	methods: {
		infiniteHandler: function ($state) {
			if (this.page < this.total_pages) {
				let req = new XMLHttpRequest();
				req.open("GET", `/get_pastes?page=${this.page + 1}&per_page=10`, true);
				req.onload = () => {
					if (req.status !== 200) {
						$state.complete();
						return;
					}
					let paginatedPastes = JSON.parse(req.responseText);
					this.page = paginatedPastes.page;
					this.total_pages = paginatedPastes.total_pages;
					this.pastes.push(...paginatedPastes.results);
					$state.loaded();
				};
				req.send();
			} else {
				$state.complete();
			}
		},
		upload: function (_event) {
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
				filename: this.title_input,
				content: this.content_input,
			}));
		},
		select: function (event) {
			let el = event.target;
			if (document.body.createTextRange) {
				const range = document.body.createTextRange();
				range.moveToElementText(el);
				range.select();
			} else if (window.getSelection) {
				const selection = window.getSelection();
				const range = document.createRange();
				range.selectNodeContents(el);
				selection.removeAllRanges();
				selection.addRange(range);
			} else {
				console.log("failed to select");
			}
		},
	},

	watch: {
		pastes: function (_value) {
			this.pastes.forEach(paste => {
				if (!paste.rendered) {
					console.log("rendering id: " + paste.id);
					paste.rendered = hljs.highlightAuto(paste.content).value;
				}
			})
		},
	},

	template: `
<div class="full_height_flex_container">
	<div class="magic">
		<div id="pastes">
			<section id="paste_form">
				<label>Title: </label><input type="text" v-model="title_input">
				<textarea 
					name="content"
					id="content_input"
					placeholder="Paste content here..."
					rows="20"
					v-model="content_input"
					required></textarea>
				<input type="button" value="Upload" v-on:click="upload">
			</section>
			<section v-for="paste in pastes" class="paste" v-on:dblclick="select">
				<div class="paste_title_bar">
					<a class="paste_title" :href="'/paste/' + paste.id">{{ paste.filename }}</a>
					<a :href="'/raw/' + paste.id">[Raw]</a>
				</div>
				<div class="paste_content hljs">
					<pre v-html="paste.rendered">
					</pre>
				</div>
				
			</section>
			
			<infinite-loading @infinite="infiniteHandler"></infinite-loading>
		</div>
	</div>
</div>
	`,
});

Vue.component("info", {
	template: `
<div id="info_tab">
	<img src="subs.png" alt="Substitutions">
	<img src="timetable.png" alt="Timetable">	
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
		let color = localStorage.getItem("color");
		if (nick !== null) {
			this.nick = nick;
			this.$emit("connect", {nick: this.nick, color: color || ""});
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
		class="chat_input"
		v-on:keyup.enter="$emit('connect', {nick, color: ''})">
</div>`,
});

const MsgType = {
	Connected: "Connected",
	Ping: "Ping",
	Message: "Message",
	Paste: "Paste",
	NickChange: "NickChange",
	ColorChange: "ColorChange",
};

Vue.component("chat", {
	props: ["user", "messages"],

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
		send: function () {
			if (this.user_msg.startsWith("/")) {
				this.send_cmd();
			} else {
				this.send_msg();
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
		send_cmd: function () {
			let xhr = new XMLHttpRequest();
			xhr.open("POST", "/send_cmd", true);
			xhr.onload = () => {
				this.user_msg = "";
				if (xhr.status !== 200) {
					console.log("request failed");
				}
			};
			xhr.setRequestHeader("content-type", "application/json");

			let msg_split = this.user_msg.split(" ");
			let cmd = msg_split[0].charAt(1).toUpperCase() + msg_split[0].slice(2);

			let payload = {};
			payload[cmd] = msg_split.slice(1).join(" ");

			let payload_json = JSON.stringify(payload);
			console.log(`Payload: ${payload_json}`);

			xhr.send(payload_json);
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
	<section class="full_height_flex_container" ref="messages">
		<div class="magic">
			<div class="message" v-for="msg in messages">
				<span>[{{ msg.time | time }}] </span><span v-bind:style="{ color: msg.custom_nick_color || 'var(--default-nick-color)' }">{{ msg.nick }}</span>: <span class="msg_content">{{ msg.msg }}</span>
			</div>
		</div>
	</section>
	<section id="input">
		<label for="msg_input" v-bind:style="{ color: user.color || 'var(--default-nick-color)' }">{{ user.nick }}</label>
		<input 
			type="text"
			v-model="user_msg"
			id="msg_input"
			class="chat_input"
			v-on:keyup.enter="send()">
	</section>
</div>
`,
});

let onload = () => (new Vue({
	el: "#app",

	data: {
		current_tab: "chat",
		tabs: ["chat", "pastes", "info"],

		eventSource: null,
		connected: false,
		has_unread_msg: false,

		user: {
			nick: "",
			color: "",
		},
		messages: [],
		pastes: [],
	},

	computed: {
		tab_props: function () {
			switch (this.current_tab) {
				case "chat":
					return {
						user: this.user,
						messages: this.messages,
					};
				case "pastes":
					return {
						user: this.user,
						pastes: this.pastes,
					};
				case "info":
					return {};
			}
		},
	},

	mounted: function () {
		window.addEventListener("focus", this.on_focus);
	},

	methods: {
		change_tab: function (tab) {
			this.current_tab = tab;
			localStorage.setItem("last_tab", tab);
		},
		handle_connect: function (user) {
			this.user = user;
			console.log("connecting as", this.user);
			this.connect();
		},
		connect: function () {
			this.eventSource = new EventSource(encodeURI(
				`/events?nick=${this.user.nick}&color=${this.user.color}`));

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

					localStorage.setItem("nick", this.user.nick);

					let last_tab = localStorage.getItem("last_tab");
					if (last_tab) {
						this.current_tab = last_tab;
					}

					this.connected = true;

					break;
				case MsgType.Ping:
					break;
				case MsgType.Message:
					this.messages.push(msg.data);
					this.notify();

					break;
				case MsgType.Paste:
					this.pastes.unshift(msg.data);
					this.notify();

					break;
				case MsgType.ColorChange:
					this.user.color = msg.data;
					localStorage.setItem("color", this.user.color);
					break;
				case MsgType.NickChange:
					this.user.nick = msg.data;
					localStorage.setItem("nick", this.user.nick);
					break;
				default:
					console.log("Unknown type: ", msg.type);
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
	},
}));

document.addEventListener("DOMContentLoaded", onload);
