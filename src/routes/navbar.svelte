<script>
 import homeIcon from "$lib/images/icons8-home.svg";
 import chatIcon from "$lib/images/icons8-chat-96.png";
 import assignmentsIcon from "$lib/images/icons8-rectangle-100.png"
</script>

<div class="navbar">
    <div>
    <span id="title">Squads</span>
    </div>
    <div class="navbar-icons">
        <a href="/" style="line-height: 0;"><img class="icon icon-selected icon-home" src={homeIcon} alt="SVG"/></a>
        <a style="line-height: 0;"><img class="icon icon-chat" src={chatIcon} alt="SVG"/></a>
        <a style="line-height: 0;"><img class="icon icon-assignments" src={assignmentsIcon} alt="SVG"/></a>
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
 .icon-selected{
    margin-bottom:2px;
    border-bottom:2px solid white;
 }
 .icon{
   cursor: pointer;
 }
 .icon-home{
   width:31.5px;
    height:31.5px;
 }
 .icon-chat{
    width:31px;
    height:31px;
    margin-top:3.5px;
 }
.icon-assignments{
    width:31px;
    height:31px;
 }
 .navbar-icons{
    position:absolute;
    right: 9px;
    display: flex;
    align-items: center;
    gap: 9px;
 }

 .navbar a {
    color: white; /* White text color */
    text-decoration: none; /* Remove underline */
 }
 .navbar a:hover {
    color: black; /* Change text color on hover */
 }
</style>


