import markdownIt from 'https://cdn.jsdelivr.net/npm/markdown-it@14.1.0/+esm';

async function postPrompt(promptMessage) {
  console.log(promptMessage)
  const resp = await fetch("/prompt", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      question: promptMessage,
    })
  })
  const responseText = await resp.text();
  return markdownIt().render(responseText)
}

function addLoader(element) {
  const loader = document.createElement('span');
  loader.setAttribute("aria-busy", "true")
  loader.id = "loader";
  loader.textContent = "🤖 is typing...";
  element.appendChild(loader)
}

function removeLoader(element) {
  const child = document.getElementById("loader");
  element.removeChild(child);
}

window.onload = async function () {
  const promptForm = document.getElementById("prompt-form");

  promptForm.addEventListener('submit', async (event) => {
    event.preventDefault();
    const promptElem = document.getElementById("prompt");
    const promptMessage = promptElem.value;
    const messagesDiv = document.getElementById("messages");
    const authorPrompt = document.createElement('p');
    const robotPrompt = document.createElement('p');
    authorPrompt.textContent = `You: ${promptMessage}`;
    messagesDiv.appendChild(authorPrompt);
    addLoader(messagesDiv);
    promptElem.value = null;
    promptElem.disabled = true;
    const resp = await postPrompt(promptMessage);
    removeLoader(messagesDiv);
    robotPrompt.innerHTML = `🤖: ${resp}`;
    messagesDiv.appendChild(robotPrompt);
    promptElem.disabled = false;
    messagesDiv.appendChild(document.createElement("hr"));
  })
}