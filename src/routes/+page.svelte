<script lang="ts">
    import { invoke } from "@tauri-apps/api/tauri";
    import viewport from "./viewportAction";

    let teams = [];
    let greetMsg;

    let query = "";

    invoke("get_cache_data", {
        key: "teams",
    }).then((data: any) => {
        if (data != null) {
            teams = data;
        }
    });

    invoke("user_teams").then((userTeams: any) => {
        teams = userTeams.teams;
        console.log("teams: " + teams);
    });
    console.log("teams 222: " + teams);
    function authorizeTeamPicture(
        event: any,
        group_id: String,
        etag: any,
        display_name: String,
    ) {
        // Sometimes etag is null. Very weird.
        console.log("event");
        if (etag != null) {
            invoke("authorize_team_picture", {
                groupId: group_id,
                etag: etag,
                displayName: display_name,
            }).then((base64data: String) => {
                event.target.src = "data:image/png;base64," + base64data;
            });
        }
    }
</script>

<svelte:head>
    <title>Home</title>
    <meta name="description" content="Svelte demo app" />
    <link rel="preconnect" href="https://fonts.googleapis.com" />
</svelte:head>

<section>
    <div id="searchFilesDiv">
        <input
            class="searchTeams"
            bind:value={query}
            placeholder="Search teams"
        />
    </div>
    <div class="teamsSelGroup" id="teams">
        {#each teams as team}
            {#if team.displayName.toLowerCase().includes(query.toLowerCase())}
                <a class="linkPage" href="team?id={team.id}&topic={team.id}"
                    ><img
                        use:viewport
                        on:enterViewport|once={() =>
                            authorizeTeamPicture(
                                event,
                                team.teamSiteInformation.groupId,
                                team.pictureETag,
                                team.displayName,
                            )}
                        src="/loading.svg"
                        alt="pfp"
                    /><span>{team.displayName}</span></a
                >
            {/if}
        {/each}
        <div></div>
    </div>
</section>
