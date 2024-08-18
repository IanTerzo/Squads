<script>
   import homeIcon from "$lib/images/icons8-home.svg";
   import chatIcon from "$lib/images/icons8-chat-96.png";
   import arrow from "$lib/images/icons8-back-64.png";
   import assignmentsIcon from "$lib/images/icons8-rectangle-100.png"
  </script>
  
  <div class="navbar">
     <a style="line-height: 0;" onclick="history.back()"><img class="icon iconHome" src={arrow} alt="SVG"/></a>
     <a style="line-height: 0;  transform: scaleX(-1);" onclick="history.forward()"><img class="icon iconHome" src={arrow} alt="SVG"/></a>
     <div class="navbarIcons">
           <a href="/" style="line-height: 0;"><img class="icon iconSelected iconHome" src={homeIcon} alt="SVG"/></a>
           <a style="line-height: 0;"><img class="icon iconChat" src={chatIcon} alt="SVG"/></a>
           <a style="line-height: 0;"><img class="icon iconAssignments" src={assignmentsIcon} alt="SVG"/></a>
      </div>
  </div>
  