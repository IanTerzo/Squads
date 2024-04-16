<script>
 import homeIcon from "$lib/images/home_FILL1_wght400_GRAD0_opsz24.svg";
 import chatIcon from "$lib/images/chat_bubble_FILL0_wght400_GRAD0_opsz24.svg";
 import assignmentsIcon from "$lib/images/description_FILL0_wght400_GRAD0_opsz24.svg";
</script>

<div class="navbar">
    <div>
    <span id="title">Squads</span>
    </div>
    <div class="navbar-icons">
        <img class="icon" src={homeIcon} alt="SVG"/>
        <img class="icon-nonsel icon-chat" src={chatIcon} alt="SVG"/>
        <img class="icon-nonsel icon-assignments" src={assignmentsIcon} alt="SVG"/>
    </div>
</div>

<style>
 .navbar {
    display: flex;
    background-color: #333;
    position: fixed;
    width: calc(100% - 150px);
    height: 50px;
    position:absolute;
    left:50%;
    transform: translateX(-50%);
    border-radius: 10px;
    top: 30px;
    z-index: 1000;
    align-items: center
 }
 #title{
     font-size:22px;
     margin-left: 8.5px;
     color:white;
 }
 .icon{
    width:34px;
    heght:34px;
    margin-bottom:2px;
    border-bottom:2px solid white;
 }
 .icon-chat{
    width:31px;
    heght:31px;
    margin-top:3.5px;
 }
.icon-assignments{
    width:32px;
    height:32px;
 }
 .navbar-icons{
    position:absolute;
    right: 8.25px;
    display: flex;
    align-items: center;
    gap: 8.25px;
 }

 .navbar a {
    color: white; /* White text color */
    text-decoration: none; /* Remove underline */
 }
 .navbar a:hover {
    color: black; /* Change text color on hover */
 }
</style>


