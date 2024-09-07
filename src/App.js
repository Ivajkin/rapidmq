import React from 'react';
import './App.css';
import MessageMetrics from './components/MessageMetrics';
import SystemHealth from './components/SystemHealth';
import ClusterStatus from './components/ClusterStatus';
import VoiceCommands from './components/VoiceCommands';

function App() {
    return (
        <div className="App">
            <h1>RapidMQ CEO Dashboard</h1>
            <div className="dashboard">
                <div className="card">
                    <h2>Message Metrics</h2>
                    <MessageMetrics />
                </div>
                <div className="card">
                    <h2>System Health</h2>
                    <SystemHealth />
                </div>
                <div className="card">
                    <h2>Cluster Status</h2>
                    <ClusterStatus />
                </div>
                <div className="card">
                    <h2>Voice Commands</h2>
                    <VoiceCommands />
                </div>
            </div>
        </div>
    );
}

export default App;