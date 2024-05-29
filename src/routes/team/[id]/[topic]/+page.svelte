<script>
import {
    onMount
} from 'svelte';
import {
    page
} from '$app/stores';
import {
    teams,
} from '../../../app.js';

import folderIcon from "$lib/images/folder.svg";

$: id = $page.params.id;
$: topic = $page.params.topic;

let team;
let section;

var conversation = {}
var lastTopic = ""

async function getConversation(teamId, topicId) {
    try {
        const response = await fetch(`/api/teamConversation/${teamId}/${topicId}`);
        if (!response.ok) {
            throw new Error('Network response was not ok');
        }
        const data = await response.json();

        const reversedReplyChains = {};
        Object.keys(data.replyChains).reverse().forEach(key => {
            reversedReplyChains[key] = data.replyChains[key];
        });

        // Reverse the order of messages in each reply chain
        Object.keys(reversedReplyChains).forEach(key => {
            reversedReplyChains[key].messages.reverse();
        });

        console.log(reversedReplyChains)
        return reversedReplyChains;


    } catch (error) {
        console.error('There was a problem with the fetch operation:', error);
    }
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
                    const imageUrl = "/api/image/" + img.getAttribute('imageid');
                    img.setAttribute('src', imageUrl);
                }
            }

            message.content = new XMLSerializer().serializeToString(doc);
        }
    }

    return conversation;
}

let files = {}

async function collectFiles(contents){
    var found = []
    for (const item of contents) {
        if (item["FileLeafRef.Suffix"] == "") { 
            // If it is a folder
            const response = await fetch(`/api/renderListDataAsStream/${section}?filesRelativePath=${item.FileRef}`);
            if (!response.ok) {
                throw new Error('Network response was not ok');
            }
            const data = await response.json();

            found[item.FileLeafRef] = await collectFiles(data.ListData.Row)
        }
        else {
            found[item.FileLeafRef] = {"url" : item[".spItemUrl"]}
        }
    }
    return found;
}


async function loadFiles(){
    let channel = team.channels.find(channel => channel.id === topic);
    let channelIndex = team.channels.indexOf(channel)

    let filesRelativePath = team.channels[channelIndex].defaultFileSettings.filesRelativePath

    const response = await fetch(`/api/renderListDataAsStream/${section}?filesRelativePath=${filesRelativePath}`);
    if (!response.ok) {
        throw new Error('Network response was not ok');
    }
    const data = await response.json();

    files = await collectFiles(data.ListData.Row)

    console.log(files)    

}

onMount(async () => {
    team = $teams.find(team => team.id === id);
    section = team.smtpAddress.split('@')[0]

    lastTopic = topic;

    var unparsedConversation = await getConversation(id, topic)
    conversation = preparseContent(unparsedConversation)
    conversation = await parseContent(conversation)
    await loadFiles();
    
});


async function loadConversation() {

    setTimeout(async function() {
        if (topic != lastTopic) {
            lastTopic = topic;
    
            var unparsedConversation = await getConversation(id, topic)
            conversation = preparseContent(unparsedConversation)
            conversation = await parseContent(conversation)
            await loadFiles();
        }
    }, 2);

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
    {#each Object.entries($teams) as [index, team]}
        {#if team.id === id}
            <div id="teamInfo">
                <img id="teamPfp" src="/api/teamPicture/{team.teamSiteInformation.groupId}/{team.displayName}/{team.pictureETag}">
                <div>
                    <div id="teamTitle">{team.displayName}</div>
                    <div id="channelName">{team.channels[0].displayName}</div>
                </div>
            </div>
            <div id="pages">
                <span>Haldor</span>
                <span>Class Notebook</span>
                <span>Assignments</span>
            </div>
            <div class="selGroup">
                {#each team.channels as channel}
                    <a class="linkPage" on:click={loadConversation} href='../../team/{id}/{channel.id}'><span># {channel.displayName}</span></a>
                {/each}
            </div>
        {/if}
    {/each}
</section>

<section id="conversationDiv">
    {#each Object.entries(conversation) as [number, replyChain]}
        <div class="activityBox">
            <div class="postSenderInfo">
                {#if replyChain.messages[0].messageType == "Event/Call"}
                    <img class="pfpImg" width="32px" height="32x" onerror="this.src='/icons8-question-mark-100.png'" src="/icons8-video-camera-96.png">
                {:else}
                    <img class="pfpImg" width="32px" height="32x" onerror="this.src='/icons8-question-mark-100.png'" src="/api/profilePicture/{replyChain.messages[0].from}/{replyChain.messages[0].imDisplayName}">
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
                <span class="titleSpan">{replyChain.messages[0].properties['subject']}</span>
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
                                <img class="file-icon" width="18px" height="18px" src="/icons8-attachment-file-64_blue.png"/><a href="{file.fileInfo.shareUrl}" target="_blank" rel="noopener noreferrer">{file.fileName}</a>
                            </div>
                        {/each}
                    {/if}
                {:else}
                    {#if index === 1}
                        {#if replyChain.messages[0].messageType == "Event/Call"}
                            <span class="showReplies" on:click={toggleReplies} style="margin-top: 0px;">Show Replies</span>
                        {:else}
                            <span class="showReplies" on:click={toggleReplies}>Show Replies</span>
                        {/if}
                    {/if}

                    <div style="display: none" class="reply">
                        <div class="messages">
                            <div class="post-sender-info">
                                <img class="pfpImg" width="32px" height="32x" onerror="this.src='/icons8-question-mark-100.png'" src="/api/profilePicture/{message.from}/{message.imDisplayName}">
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

<section id="files">
<span># General </span>
<div class="folder">
    <div class="folderTitle"> <img src={folderIcon}> Ellära </div>
    <div class="folder">
        <span class="filesFile">ellära.pptx</span>
        <span class="filesFile" >ohmslag.pptx</span>
    </div>
</div>
<span># Annat </span>
<div class="folder">
    <div class="folderTitle"> <img src={folderIcon}> Teknik </div>
    <div class="folder">
        <span class="filesFile">teknik.pdf</span>

        <div class="folderTitle"> <img src={folderIcon}> Electromagnetic Compatibility</div>
        <div class="folder">
             <span class="filesFile">Electromagneic Fields.pptx</span>
        </div>

    </div>

</div>
{#each Object.entries(files) as [index, team]}
<span>{index}</span>
{/each}
</section>

<style>

</style>

