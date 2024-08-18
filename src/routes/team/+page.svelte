<script lang="ts">
    import { onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/tauri";
    import viewport from "../viewportAction";

    const urlParams = new URLSearchParams(window.location.search);

    const id = urlParams.get("id");
    const topic = urlParams.get("topic");

    let team = null;
    let teamPfp = "/loading.svg";
    let section;
    let webUrl;

    var conversation = null;
    let files = "";

    var lastTopic = "";

    async function loadData() {
        const data = await invoke("get_cache_data", { key: "teams" });
        team = data.find((team) => team.id === id);
        section = team.smtpAddress.split("@")[0];

        webUrl = await invoke("get_weburl");

        invoke("team_conversations", {
            teamId: id,
            topicId: topic,
        }).then(async (data: any) => {
            console.log(id);
            console.log(topic);
            conversation = await preparseContent(data.replyChains);
            //await parseImages(conversation);
        });
        if (team.pictureETag != null) {
            invoke("authorize_team_picture", {
                groupId: team.teamSiteInformation.groupId,
                etag: team.pictureETag,
                displayName: team.displayName,
            }).then((base64data: String) => {
                teamPfp = "data:image/png;base64," + base64data;
            });
        }

        for (const channel of team.channels) {
            let channelInfo = await invoke("team_channel_info", {
                groupId: team.teamSiteInformation.groupId,
                topicId: channel.id,
            });

            // TEMPORARY FIX
            let filesRelativePath =
                channelInfo.filesFolderWebUrl.replace("https://");
            let filesRelativePathSplit = filesRelativePath.split("/");
            filesRelativePathSplit.shift();
            filesRelativePath = "/" + filesRelativePathSplit.join("/");

            console.log(filesRelativePath);
            let dataSream = await invoke("render_list_data_as_stream", {
                section: section,
                filesRelativePath: filesRelativePath,
            });

            let documentLibraries = await invoke("document_libraries", {
                topicId: channel.id,
            });

            let serverFiles = "";

            if (documentLibraries[0]) {
                let serverRelativeUrl =
                    documentLibraries[0].object.serverRelativeUrl;

                let serverDataSream = await invoke(
                    "render_list_data_as_stream",
                    {
                        section: section,
                        filesRelativePath: serverRelativeUrl,
                    },
                );

                try {
                    let filesFetched = await collectFiles(serverDataSream.Row);
                    if (filesFetched.length != 0) {
                        serverFiles += '<div class="folder">';
                        serverFiles += filesFetched;
                        serverFiles += "</div>";
                    }
                } catch (err) {
                    console.log("Error while fetching! : " + err);
                }

                console.log(dataSream);
            }

            try {
                let filesFetched = await collectFiles(dataSream.Row);
                if (filesFetched.length != 0) {
                    files += `<span>${channel.displayName} </span>`;
                    files += serverFiles;
                    files += '<div class="folder">';
                    files += filesFetched;
                    files += "</div>";
                }
            } catch (err) {
                console.log("Error while fetching! : " + err);
            }
        }

        if (files == "") {
            files = " ";
        }
    }

    loadData();

    function authorizeProfilePicture(
        event: any,
        user_id: String,
        display_name: String,
    ) {
        // Sometimes etag is null. Very weird.
        invoke("authorize_profile_picture", {
            userId: user_id,
            displayName: display_name,
        }).then((base64data: String) => {
            event.target.src = "data:image/png;base64," + base64data;
        });
    }

    function cleanUpBody(node) {
        node.removeAttribute("style");
        if (node.tagName == "IMG") {
            const src = node.getAttribute("src");
            const imageId = src
                .replace("https://eu-api.asm.skype.com/v1/objects/", "")
                .replace(
                    "https://eu-prod.asyncgw.teams.microsoft.com/v1/objects/",
                    "",
                )
                .replace("/views/imgo", "");

            node.setAttribute("imageid", imageId);

            // We cannot use addeventlistener becuase the images aren't in the dom yet.
            //  Instead, we can set the loading to lazy and when the images gives an error set the correct image
            node.setAttribute("src", "/");
            node.setAttribute("loading", "lazy");
            node.setAttribute(
                "onerror",
                "this.src = '/loading.svg'; window.__TAURI__.invoke('authorize_image', {imageId: this.getAttribute('imageid')}).then((base64data) => {this.src = 'data:image/png;base64,' + base64data});",
            );
        }

        if (node.getAttribute("itemtype") == "http://schema.skype.com/Emoji") {
            node.outerHTML = `<span> ${node.alt} </span>`;
        }

        if (node.textContent == "\u00A0") {
            node.innerHTML = node.innerHTML.replace("&nbsp;", "");
        }

        for (let i = 0; i < node.children.length; i++) {
            cleanUpBody(node.children[i]);
        }
    }

    async function preparseContent(conversation) {
        conversation;
        for (const replyChain of Object.values(conversation)) {
            for (const message of replyChain.messages) {
                const parser = new DOMParser();
                const doc = parser.parseFromString(
                    message.content,
                    "text/html",
                );

                var body = doc.childNodes[0].childNodes[1];
                cleanUpBody(body);

                message.content = body.outerHTML;
            }
        }

        return conversation;
    }

    async function collectFiles(contents) {
        var found = "";
        for (const item of contents) {
            if (item["FileLeafRef.Suffix"] == "") {
                // If it is a folder
                const data = await invoke("render_list_data_as_stream", {
                    section: section,
                    filesRelativePath: item.FileRef,
                });
                found += `<div class="folderTitle"> <img src="/folder.svg"> ${item.FileLeafRef}</div>`;

                found += '<div class="folder">';

                let contents = await collectFiles(data.Row);
                found += contents;
                found += "</div>";
            } else {
                found += `<a class="filesFile" target="_blank" rel="noopener noreferrer" href="${webUrl + item["FileRef.urlencodeasurl"]}">${item.FileLeafRef}</a>`;
            }
        }
        return found;
    }

    async function loadConversation(channelID) {
        conversation = null;

        // channelId is the same as topic but it updates earlier since topic is reactive and changes with the href
        invoke("team_conversations", {
            teamId: id,
            topicId: channelID,
        }).then(async (data: any) => {
            conversation = await preparseContent(data.replyChains);
            //await parseImages(conversation);
        });
    }

    function toggleReplies(content) {
        const element = event.target;
        const replies = element.parentNode.querySelectorAll(".reply");
        if (
            replies[0].style.display == "none" ||
            replies[0].style.display == ""
        ) {
            element.textContent = "Hide Replies";
            for (const reply of replies) {
                reply.style.display = "block";
            }
        } else {
            element.textContent = "Show Replies";
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

<section id="teamInfoSection">
    {#if team != null}
        <div id="teamInfo">
            <img id="teamPfp" src={teamPfp} />
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
                <a
                    class="linkPage"
                    on:click={() => loadConversation(channel.id)}
                    href="../../team/{id}/{channel.id}"
                    ><span># {channel.displayName}</span></a
                >
            {/each}
        </div>
    {/if}
</section>

<section id="conversationDiv">
    {#if conversation != null}
        {#each conversation as replyChain}
            <div class="activityBox">
                <div class="postSenderInfo">
                    {#if replyChain.messages[0].messageType == "Event/Call"}
                        <img
                            class="pfpImg"
                            width="32px"
                            height="32x"
                            alt="pfp"
                            onerror="this.src='/icons8-question-mark-100.png'"
                            src="/icons8-video-camera-96.png"
                        />
                    {:else}
                        <img
                            use:viewport
                            on:enterViewport|once={() =>
                                authorizeProfilePicture(
                                    event,
                                    replyChain.messages[0].from,
                                    replyChain.messages[0].imDisplayName,
                                )}
                            class="pfpImg"
                            width="32px"
                            height="32x"
                            alt="pfp"
                            onerror="this.src='/icons8-question-mark-100.png'"
                            src="/loading.svg"
                        />
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

                {#if replyChain.messages[0].properties["subject"]}
                    <span class="titleSpan"
                        >{replyChain.messages[0].properties["subject"]}</span
                    >
                {/if}

                {#each replyChain.messages as message, index}
                    {#if index === 0}
                        {#if message.properties["systemdelete"] || message.properties["deletetime"]}
                            <i><span>Deleted Message</span></i>
                        {:else}
                            <div id="content">{@html message.content}</div>
                        {/if}
                        {#if replyChain.messages[0].properties["files"] && replyChain.messages[0].properties["files"] != "[]"}
                            {#each JSON.parse(replyChain.messages[0].properties["files"]) as file}
                                <div class="file">
                                    <img
                                        class="file-icon"
                                        width="18px"
                                        height="18px"
                                        src="/icons8-attachment-file-64_blue.png"
                                        alt="icon"
                                    />
                                    <a
                                        href={file.fileInfo.fileUrl}
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        >{file.fileName}</a
                                    >
                                </div>
                            {/each}
                        {/if}
                    {:else}
                        {#if index === 1}
                            {#if replyChain.messages[0].messageType == "Event/Call"}
                                <span
                                    class="showReplies"
                                    on:click={toggleReplies}
                                    style="margin-top: 0px;">Show Replies</span
                                >
                            {:else}
                                <span
                                    class="showReplies"
                                    on:click={toggleReplies}>Show Replies</span
                                >
                            {/if}
                        {/if}

                        <div style="display: none" class="reply">
                            <div class="messages">
                                <div class="post-sender-info">
                                    <img
                                        use:viewport
                                        on:enterViewport|once={() =>
                                            authorizeProfilePicture(
                                                event,
                                                message.from,
                                                message.imDisplayName,
                                            )}
                                        class="pfpImg"
                                        width="32px"
                                        height="32x"
                                        alt="pfp"
                                        onerror="this.src='/icons8-question-mark-100.png'"
                                        src="/loading.svg"
                                    />
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

                                {#if message.properties["systemdelete"] || message.properties["deletetime"]}
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
    {/if}
</section>

<section id="files">
    <div id="searchFilesDiv">
        <input class="searchFiles" placeholder="Search files" />
    </div>
    <div id="filesContainer">
        {#if files != ""}
            {@html files}
        {:else}
            <span id="fetchingFiles">Fetching...</span>
        {/if}
    </div>
</section>

<style>
</style>
