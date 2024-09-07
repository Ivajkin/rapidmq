import React, { useEffect } from 'react';
import annyang from 'annyang';

const VoiceCommands = () => {
    useEffect(() => {
        if (annyang) {
            const commands = {
                'show system health': () => {
                    alert('System health: CPU usage 30%, Memory usage 50%, Disk usage 20%');
                },
                'show cluster status': () => {
                    alert('Cluster status: 5 active nodes, 1 inactive node');
                },
                'create queue *name': (name) => {
                    fetch(`/queue/${name}`, { method: 'POST' })
                        .then(response => response.text())
                        .then(data => {
                            alert(data);
                        });
                },
                'publish message *message to queue *name': (message, name) => {
                    fetch('/publish', {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({ queue_name: name, message: message })
                    })
                        .then(response => response.text())
                        .then(data => {
                            alert(data);
                        });
                },
                'consume message from queue *name': (name) => {
                    fetch(`/consume/${name}`)
                        .then(response => response.json())
                        .then(data => {
                            alert(`Consumed message: ${data.content}`);
                        })
                        .catch(() => alert('No messages in queue'));
                }
            };

            annyang.addCommands(commands);
            annyang.start();
        }
    }, []);

    return (
        <div>
            <button onClick={() => annyang.start()}>Start Voice Command</button>
            <p>Try saying "show system health" or "show cluster status".</p>
        </div>
    );
};

export default VoiceCommands;