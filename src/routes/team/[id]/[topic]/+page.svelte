<script>
    import { page } from '$app/stores';
    import { teams} from '../../../app.js';
    
    console.log($teams)
    let id;
    $: id = $page.params.id;

    let topic;
    $: topic = $page.params.topic;
 
</script>
   
   <div>
    {id}
    {topic}

   </div>