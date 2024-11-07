pub const INDEX_DATA: &str = r#"
<!doctype html>
<html lang="en">
  <title>WTransport-Example</title>
  <meta charset="utf-8">
  <script src="client.js"></script>
  <link rel="stylesheet" href="style.css">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <body>

    <h1>WTransport Example</h1>

    <div>
      <h2>Establish WebTransport connection</h2>
      <div class="input-line">
        <label for="url">URL:</label>
        <input type="text" name="url" id="url" value="https://localhost:${WEBTRANSPORT_PORT}/">
        <input type="button" id="connect" value="Connect" onclick="connect()">
      </div>
    </div>

    <div>
      <h2>Send data over WebTransport</h2>
      <form name="sending">
        <textarea name="data" id="data"></textarea>
        <div>
          <input type="radio" name="sendtype" value="datagram" id="datagram" checked>
          <label for="datagram">Send a datagram</label>
        </div>
        <div>
          <input type="radio" name="sendtype" value="unidi" id="unidi-stream">
          <label for="unidi-stream">Open a unidirectional stream</label>
        </div>
        <div>
          <input type="radio" name="sendtype" value="bidi" id="bidi-stream">
          <label for="bidi-stream">Open a bidirectional stream</label>
        </div>
        <input type="button" id="send" name="send" value="Send data" disabled onclick="sendData()">
      </form>
    </div>

    <div>
      <h2>Event log</h2>
      <ul id="event-log">
      </ul>
    </div>

  </body>
</html>
"#;

pub const STYLE_DATA: &str = r#"
body {
  font-family: sans-serif;
}

h1 {
  margin: 0 auto;
  width: fit-content;
}

h2 {
  border-bottom: 1px dotted #333;
  font-size: 120%;
  font-weight: normal;
  padding-bottom: 0.2em;
  padding-top: 0.5em;
}

code {
  background-color: #eee;
}

input[type=text], textarea {
  font-family: monospace;
}

#top {
  display: flex;
  flex-direction: row-reverse;
  flex-wrap: wrap;
  justify-content: center;
}

#explanation {
  border: 1px dotted black;
  font-size: 90%;
  height: fit-content;
  margin-bottom: 1em;
  padding: 1em;
  width: 13em;
}

#tool {
  flex-grow: 1;
  margin: 0 auto;
  max-width: 26em;
  padding: 0 1em;
  width: 26em;
}

.input-line {
  display: flex;
}

.input-line input[type=text] {
  flex-grow: 1;
  margin: 0 0.5em;
}

textarea {
  height: 3em;
  width: 100%;
}

#send {
  margin-top: 0.5em;
  width: 15em;
}

#event-log {
  border: 1px dotted black;
  font-family: monospace;
  height: 12em;
  overflow: scroll;
  padding-bottom: 1em;
  padding-top: 1em;
}

.log-error {
  color: darkred;
}

#explanation ul {
  padding-left: 1em;
}
"#;

pub const CLIENT_DATA: &str = r#"
// Adds an entry to the event log on the page, optionally applying a specified
// CSS class.

const HASH = new Uint8Array(${CERT_DIGEST});

let currentTransport, streamNumber, currentTransportDatagramWriter;

// "Connect" button handler.
async function connect() {
  const url = document.getElementById('url').value;
  try {
    var transport = new WebTransport(url, { serverCertificateHashes: [ { algorithm: "sha-256", value: HASH.buffer } ] } );
    addToEventLog('Initiating connection...');
  } catch (e) {
    addToEventLog('Failed to create connection object. ' + e, 'error');
    return;
  }

  try {
    await transport.ready;
    addToEventLog('Connection ready.');
  } catch (e) {
    addToEventLog('Connection failed. ' + e, 'error');
    return;
  }

  transport.closed
      .then(() => {
        addToEventLog('Connection closed normally.');
      })
      .catch(() => {
        addToEventLog('Connection closed abruptly.', 'error');
      });

  currentTransport = transport;
  streamNumber = 1;
  try {
    currentTransportDatagramWriter = transport.datagrams.writable.getWriter();
    addToEventLog('Datagram writer ready.');
  } catch (e) {
    addToEventLog('Sending datagrams not supported: ' + e, 'error');
    return;
  }
  readDatagrams(transport);
  acceptUnidirectionalStreams(transport);
  document.forms.sending.elements.send.disabled = false;
  document.getElementById('connect').disabled = true;
}

// "Send data" button handler.
async function sendData() {
  let form = document.forms.sending.elements;
  let encoder = new TextEncoder('utf-8');
  let rawData = sending.data.value;
  let data = encoder.encode(rawData);
  let transport = currentTransport;
  try {
    switch (form.sendtype.value) {
      case 'datagram':
        await currentTransportDatagramWriter.write(data);
        addToEventLog('Sent datagram: ' + rawData);
        break;
      case 'unidi': {
        let stream = await transport.createUnidirectionalStream();
        let writer = stream.getWriter();
        await writer.write(data);
        await writer.close();
        addToEventLog('Sent a unidirectional stream with data: ' + rawData);
        break;
      }
      case 'bidi': {
        let stream = await transport.createBidirectionalStream();
        let number = streamNumber++;
        readFromIncomingStream(stream.readable, number);

        let writer = stream.writable.getWriter();
        await writer.write(data);
        await writer.close();
        addToEventLog(
            'Opened bidirectional stream #' + number +
            ' with data: ' + rawData);
        break;
      }
    }
  } catch (e) {
    addToEventLog('Error while sending data: ' + e, 'error');
  }
}

// Reads datagrams from |transport| into the event log until EOF is reached.
async function readDatagrams(transport) {
  try {
    var reader = transport.datagrams.readable.getReader();
    addToEventLog('Datagram reader ready.');
  } catch (e) {
    addToEventLog('Receiving datagrams not supported: ' + e, 'error');
    return;
  }
  let decoder = new TextDecoder('utf-8');
  try {
    while (true) {
      const { value, done } = await reader.read();
      if (done) {
        addToEventLog('Done reading datagrams!');
        return;
      }
      let data = decoder.decode(value);
      addToEventLog('Datagram received: ' + data);
    }
  } catch (e) {
    addToEventLog('Error while reading datagrams: ' + e, 'error');
  }
}

async function acceptUnidirectionalStreams(transport) {
  let reader = transport.incomingUnidirectionalStreams.getReader();
  try {
    while (true) {
      const { value, done } = await reader.read();
      if (done) {
        addToEventLog('Done accepting unidirectional streams!');
        return;
      }
      let stream = value;
      let number = streamNumber++;
      addToEventLog('New incoming unidirectional stream #' + number);
      readFromIncomingStream(stream, number);
    }
  } catch (e) {
    addToEventLog('Error while accepting streams: ' + e, 'error');
  }
}

async function readFromIncomingStream(stream, number) {
  let decoder = new TextDecoderStream('utf-8');
  let reader = stream.pipeThrough(decoder).getReader();
  try {
    while (true) {
      const { value, done } = await reader.read();
      if (done) {
        addToEventLog('Stream #' + number + ' closed');
        return;
      }
      let data = value;
      addToEventLog('Received data on stream #' + number + ': ' + data);
    }
  } catch (e) {
    addToEventLog(
        'Error while reading from stream #' + number + ': ' + e, 'error');
    addToEventLog('    ' + e.message);
  }
}

function addToEventLog(text, severity = 'info') {
  let log = document.getElementById('event-log');
  let mostRecentEntry = log.lastElementChild;
  let entry = document.createElement('li');
  entry.innerText = text;
  entry.className = 'log-' + severity;
  log.appendChild(entry);

  // If the most recent entry in the log was visible, scroll the log to the
  // newly added element.
  if (mostRecentEntry != null &&
      mostRecentEntry.getBoundingClientRect().top <
          log.getBoundingClientRect().bottom) {
    entry.scrollIntoView();
  }
}
"#;
