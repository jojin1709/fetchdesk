chrome.runtime.onInstalled.addListener(() => {
  chrome.contextMenus.create({
    id: "sendToFetchDesk",
    title: "Send Link to FetchDesk",
    contexts: ["link", "video", "audio"]
  });
});

chrome.contextMenus.onClicked.addListener((info, tab) => {
  if (info.menuItemId === "sendToFetchDesk") {
    const url = info.linkUrl || info.srcUrl || tab.url;
    if (url) {
      sendToFetchDesk(url);
    }
  }
});

function sendToFetchDesk(url) {
  fetch("http://127.0.0.1:8382/download", {
    method: "POST",
    headers: {
      "Content-Type": "application/json"
    },
    body: JSON.stringify({ url: url })
  })
  .then(response => response.json())
  .then(data => {
    console.log("Success:", data);
  })
  .catch(error => {
    console.error("Error connecting to FetchDesk webhook:", error);
  });
}
