<script>
import { onDestroy, onMount } from 'svelte';
import { page } from '$app/stores';
import {profilePicture, authorizeImage, teams, getConversation } from '../../../app.js';

$: id = $page.params.id;
$: topic = $page.params.topic;
var conversation = {}
var lastTopic = ""

onMount(async () => {
    lastTopic = topic;
    var unparsedConversation = await getConversation(id,topic)
    conversation = preparseContent(unparsedConversation)
    conversation = await parseContent(conversation)
});


async function loadConversation(){
 
    setTimeout(async function() {
        if (topic != lastTopic){
            var unparsedConversation = await getConversation(id,topic)
            conversation = preparseContent(unparsedConversation)
            conversation = await parseContent(conversation)

            lastTopic = topic;
        }
    }, 5);

}

function preparseContent(conversation) {
    for (const replyChain of Object.values(conversation)) {
        for (const message of replyChain.messages) {
            const parser = new DOMParser();
            const doc = parser.parseFromString(message.content, 'text/html');
    
            const paragraphs = doc.querySelectorAll('p');
            paragraphs.forEach(p => {
                
                
                if (p.textContent === '\u00A0') {
                    p.innerHTML = p.innerHTML.replace("&nbsp;", "")
                }
                if (p.innerHTML === "") {
                    p.parentNode.removeChild(p);
                }
            });

            const images = doc.querySelectorAll('[itemtype="http://schema.skype.com/AMSImage"]');

            for (const img of images) {
                const imageUrl = "/loading.png";
                img.setAttribute('src', imageUrl);
            }

            message.content = new XMLSerializer().serializeToString(doc);
        }
    }

    return conversation;
}



async function parseContent(conversation) {
    console.log(profilePicture("hi"))
    for (const replyChain of Object.values(conversation)) {
        for (const message of replyChain.messages) {
            const parser = new DOMParser();
            const doc = parser.parseFromString(message.content, 'text/html');
    
            const images = doc.querySelectorAll('[itemtype="http://schema.skype.com/AMSImage"]');

            for (const img of images) {
                if (img.attributes.id){
                const imageUrl = await authorizeImage(img.getAttribute('id').substring(2)); // Remove the x_ part of the id;
                img.setAttribute('src', imageUrl);
            }
            }

            message.content = new XMLSerializer().serializeToString(doc);
        }
    }

    return conversation;
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
            <div id="teamTitle"> {team.name} </div>
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
                    <img class="pfp-img" width="32px" height="32x" alt="error" src="">
                    {#if !replyChain.messages[0].imDisplayName}
                        <span>Uknown User</span>
                    {:else}
                    <span>{replyChain.messages[0].imDisplayName}</span>    
                    {/if}
                                    
                </div>
                
                {#if replyChain.messages[0].properties['subject']}
                    <span class="titlespan">{replyChain.messages[0].properties['subject']}</span>
                {/if}
                
                {#each replyChain.messages as message, index}
                {#if index === 0}
                {#if message.content == '<html xmlns="http://www.w3.org/1999/xhtml"><head></head><body>null</body></html>'}
                <span>Deleted Message</span>
            {:else}
            <div id="content">{@html message.content}</div>
            {/if}
            {#if replyChain.messages[0].properties['files'] && replyChain.messages[0].properties['files'] != "[]"}
                <div class="file">
                    <img class="file-icon" width="18px" height="18px" src="/icons8-attachment-file-64_blue.png"/><a>{JSON.parse(replyChain.messages[0].properties['files'])[0].fileName}</a>
                </div>
            {/if}
                   
                    
                   
                {:else}
                    {#if index === 1}
                        <span class="show-replies"> Hide Replies</span>
                    {/if}
                <div class="replies">
                    
                    <div class="reply">
                    <div class="messages">
                    <div class="post-sender-info">
                    <img class="pfp-img" width="32px" height="32x" src="https://upload.wikimedia.org/wikipedia/commons/thumb/1/15/Cat_August_2010-4.jpg/1200px-Cat_August_2010-4.jpg">
                    <span>John Toe</span>
                
                    </div>
                    <span>{@html message.content} </span>
                    </div>
                    </div>
                </div>
                {/if}
                
                
                {/each}
            </div>
            {/each}


</section>
<style>
#teamInfo{
display:flex;
align-items: center;
gap:10px;
margin-bottom:20px;
}
#teamPfp{
border-radius:4px;
}

.file{
    cursor: pointer;
    background-color: #424242;
    width: fit-content;
    padding: 3.5px;
    padding-right: 5px;
    padding-left: 5px;
    border-radius: 4px;
   
}

.file-icon{
    margin-right: 3px;
    transform: translateY(1.3px);
}

#channelName{
    color:white;
	font-size: 16px
}

#pages{
    display: flex;
    flex-direction: column;
    color: white;
    gap: 6px;
    cursor: pointer;
    margin-top: 15px;
    margin-bottom: 18px;
}
#teamTitle{
    color:white;
	position:relative;
	font-size: 16px;
    font-weight: 600;

}

#conversationDiv {
    color: white;
    display: flex;
    flex-direction: column-reverse; /* Reverse the order of items */
    overflow-y: scroll;

    height: 87vh;
    scrollbar-width: none;
}
:global([itemtype="http://schema.skype.com/Mention"]) {
    color: #6698d9;
    /* Add more styles as needed */
}
</style>

