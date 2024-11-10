<script lang="ts">
    import { onMount } from 'svelte';

    let message = 'Loading...';

    onMount(async () => {
        try {
            await new Promise(r => setTimeout(r, 2000));
            const response = await fetch('/api/hello');
            const data = await response.json();
            message = data.message;
        } catch (error) {
            message = 'Error loading message';
            console.error('Error:', error);
        }
    });
</script>

<svelte:head>
    <title>Home</title>
    <meta name="description" content="Svelte + Rust application" />
</svelte:head>

<div class="container">
    <h1>Welcome to SvelteKit</h1>
    <p>{message}</p>
</div>

<style>
    .container {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        height: 100%;
        text-align: center;
    }

    h1 {
        color: #ff3e00;
        font-size: 4em;
        font-weight: 100;
    }

    p {
        color: #666;
        font-size: 1.2em;
        margin: 1em 0;
    }
</style>