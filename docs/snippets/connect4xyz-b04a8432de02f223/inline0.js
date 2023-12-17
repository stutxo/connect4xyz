
export async function pub_key() {
    if (typeof window.nostr !== 'undefined') {
        try {
            const encoder = new TextEncoder();
            const publicKey = await window.nostr.getPublicKey();
            const view = encoder.encode(publicKey);
            console.log(view);
            return view;
        } catch (error) {
            // Handle the error when the popup is closed or any other error
            console.error('Error occurred:', error);
            // Return null or handle it in a way that does not crash your app
            return null;
        }
    } else {
        console.error('window.nostr is not available');
        return null;
    }
}
