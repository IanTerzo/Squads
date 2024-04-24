<script>
import {
    onDestroy,
    onMount
} from 'svelte';
import {
    page
} from '$app/stores';
import {
    profilePicture,
    authorizeImage,
    teams,
    getConversation
} from '../../../app.js';

$: id = $page.params.id;
$: topic = $page.params.topic;
var conversation = {}
var lastTopic = ""

onMount(async () => {
    lastTopic = topic;
    var unparsedConversation = await getConversation(id, topic)
    conversation = preparseContent(unparsedConversation)
    conversation = await parseContent(conversation)
});


async function loadConversation() {

    setTimeout(async function() {
        if (topic != lastTopic) {
            var unparsedConversation = await getConversation(id, topic)
            conversation = preparseContent(unparsedConversation)
            conversation = await parseContent(conversation)

            lastTopic = topic;
        }
    }, 5);

}

function cleanUpBody(node) {

    node.removeAttribute('style');

    if (node.tagName == "IMG") {
        const src = node.getAttribute("src")
        const imageId = src.replace("https://eu-api.asm.skype.com/v1/objects/", "").replace("https://eu-prod.asyncgw.teams.microsoft.com/v1/objects/", "").replace("/views/imgo", "")

        //node.setAttribute('src', "/loading.png");
        node.setAttribute('imageid', imageId);

    }

    if (node.getAttribute("itemtype") == "http://schema.skype.com/Emoji") {
        node.outerHTML = `<span> ${node.alt} </span>`
    }

    if (node.textContent == '\u00A0') {
        node.innerHTML = node.innerHTML.replace("&nbsp;", "")
    }

    for (let i = 0; i < node.children.length; i++) {
        cleanUpBody(node.children[i]);
    }
}


function preparseContent(conversation) {
    for (const replyChain of Object.values(conversation)) {
        for (const message of replyChain.messages) {
            const parser = new DOMParser();
            const doc = parser.parseFromString(message.content, 'text/html');

            var body = doc.childNodes[0].childNodes[1];
            cleanUpBody(body);

            message.content = body.outerHTML
        }
    }

    return conversation;
}

async function parseContent(conversation) {
    for (const replyChain of Object.values(conversation)) {
        for (const message of replyChain.messages) {
            const parser = new DOMParser();
            const doc = parser.parseFromString(message.content, 'text/html');

            const images = doc.querySelectorAll('[itemtype="http://schema.skype.com/AMSImage"]');

            for (const img of images) {
                if (img.attributes.imageid) {
                    const imageUrl = await authorizeImage(img.getAttribute('imageid'));
                    img.setAttribute('src', imageUrl);
                }
            }

            message.content = new XMLSerializer().serializeToString(doc);
        }
    }

    return conversation;
}

function toggleReplies(content) {
    const element = event.target;
    const replies = element.parentNode.querySelectorAll(".reply")
    if (replies[0].style.display == "none" || replies[0].style.display == "") {
        element.textContent = "Hide Replies"
        for (const reply of replies) {
            reply.style.display = "block";
        }
    } else {
        element.textContent = "Show Replies"
        for (const reply of replies) {
            reply.style.display = "none";
        }
    }
}
</script>

<svelte:head>
	<title>Home</title>
	<meta name="description" content="Svelte demo app" />
</svelte:head>

<section>
    {#each Object.entries($teams) as [teamId, team]}
        {#if teamId === id}
            <div id="teamInfo">
                <img id="teamPfp" width="50px" height="50px" src="https://upload.wikimedia.org/wikipedia/commons/thumb/1/15/Cat_August_2010-4.jpg/1200px-Cat_August_2010-4.jpg">
                <div>
                    <div id="teamTitle">{team.name}</div>
                    <div id="channelName">General</div>
                </div>
            </div>
            <div id="pages">
                <span>Haldor</span>
                <span>Class Notebook</span>
                <span>Assignments</span>
            </div>
            <div class="selgroup">
                {#each team.topics as topic}
                    <a class="linkpage" on:click={loadConversation} href='../../team/{id}/{topic.id}'><span># {topic.name}</span></a>
                {/each}
            </div>
        {/if}
    {/each}
</section>

<section id="conversationDiv">
    {#each Object.entries(conversation) as [number, replyChain]}
        <div class="activity-box">
            <div class="post-sender-info">
                {#if replyChain.messages[0].messageType == "Event/Call"}
                    <img class="pfp-img" width="32px" height="32x" onerror="this.src='/icons8-question-mark-100.png'" src="/icons8-video-camera-96.png">
                {:else}
                    <img class="pfp-img" width="32px" height="32x" onerror="this.src='/icons8-question-mark-100.png'" src="http://localhost:5102/profilePicture/{replyChain.messages[0].from}/{replyChain.messages[0].imDisplayName}">
                {/if}

                {#if !replyChain.messages[0].imDisplayName}
                    {#if replyChain.messages[0].messageType == "Event/Call"}
                        <span><b>Meeting Started</b></span>
                    {:else}
                        <span>Unkown User</span>
                    {/if}
                {:else}
                    <span>{replyChain.messages[0].imDisplayName}</span>
                {/if}
            </div>

            {#if replyChain.messages[0].properties['subject']}
                <span class="titlespan">{replyChain.messages[0].properties['subject']}</span>
            {/if}

            {#each replyChain.messages as message, index}
                {#if index === 0}
                    {#if message.properties['systemdelete'] || message.properties['deletetime']}
                        <i><span>Deleted Message</span></i>
                    {:else}
                        <div id="content">{@html message.content}</div>
                    {/if}
                    {#if replyChain.messages[0].properties['files'] && replyChain.messages[0].properties['files'] != "[]"}
                        {#each JSON.parse(replyChain.messages[0].properties['files']) as file}
                            <div class="file">
                                <img class="file-icon" width="18px" height="18px" src="/icons8-attachment-file-64_blue.png"/><a href="{file.fileInfo.fileUrl}" download>{file.fileName}</a>
                            </div>
                        {/each}
                    {/if}
                {:else}
                    {#if index === 1}
                        {#if replyChain.messages[0].messageType == "Event/Call"}
                            <span class="show-replies" on:click={toggleReplies} style="margin-top: 0px;">Show Replies</span>
                        {:else}
                            <span class="show-replies" on:click={toggleReplies}>Show Replies</span>
                        {/if}
                    {/if}

                    <div style="display: none" class="reply">
                        <div class="messages">
                            <div class="post-sender-info">
                                <img class="pfp-img" width="32px" height="32x" onerror="this.src='/icons8-question-mark-100.png'" src="http://localhost:5102/profilePicture/{message.from}/{message.imDisplayName}">
                                {#if !message.imDisplayName}
                                    {#if replyChain.messages[0].messageType == "Event/Call"}
                                        <span><b>Meeting Ended</b></span>
                                    {:else}
                                        <span>Unkown User</span>
                                    {/if}
                                {:else}
                                    <span>{message.imDisplayName}</span>
                                {/if}
                            </div>

                            {#if message.properties['systemdelete'] || message.properties['deletetime']}
                                <i><span>Deleted Message</span></i>
                            {:else}
                                <span>{@html message.content}</span>
                            {/if}
                        </div>
                    </div>
                {/if}
            {/each}
        </div>
    {/each}
</section>
<style>
#teamInfo {
	display: flex;
	align-items: center;
	gap: 10px;
	margin-bottom: 20px;
}

#teamPfp {
	border-radius: 4px;
}

#channelName {
	color: white;
	font-size: 16px
}

#pages {
	display: flex;
	flex-direction: column;
	color: white;
	gap: 6px;
	cursor: pointer;
	margin-top: 15px;
	margin-bottom: 18px;
}

#teamTitle {
	color: white;
	position: relative;
	font-size: 16px;
	font-weight: 600;
}

#conversationDiv {
	color: white;
	display: flex;
	flex-direction: column-reverse;
	/* Reverse the order of items */
	overflow-y: scroll;

	height: 87vh;
	scrollbar-width: none;
}
</style>

