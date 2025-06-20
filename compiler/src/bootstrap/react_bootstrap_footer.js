if (typeof window !== 'undefined') {
    const container = document.getElementById('root');
    if (container) {
        hydrateRoot(container, <Index/>);
    }
}