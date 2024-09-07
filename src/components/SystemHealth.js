import React from 'react';
import { Doughnut } from 'react-chartjs-2';

const SystemHealth = () => {
    const data = {
        labels: ['CPU Usage', 'Memory Usage', 'Disk Usage'],
        datasets: [{
            data: [30, 50, 20],
            backgroundColor: [
                'rgba(255, 99, 132, 0.8)',
                'rgba(54, 162, 235, 0.8)',
                'rgba(255, 206, 86, 0.8)'
            ]
        }]
    };

    return <Doughnut data={data} />;
};

export default SystemHealth;