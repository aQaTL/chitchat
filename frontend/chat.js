window.onload = () => (new Vue({
	el: "#app",

	data: {
		wsConn: null,
		msg: "Hello, World",
	},
	
	mounted: function() {
		this.connect();
	},

	methods: {
		connect: function() {
			let url = `ws://${window.location.hostname}:${window.location.port}/ws`;
			console.log(`Connecting to ${url}`);
			this.wsConn = new WebSocket(url);
			this.wsConn.onerror = function(event) {
				this.msg = "Error";
			};
			this.wsConn.onclose = function(event) {
				this.msg = "Closed";
			};
			this.wsConn.onopen = function(event) {
				this.msg = "Connected";
			};
		},
	},
}));
