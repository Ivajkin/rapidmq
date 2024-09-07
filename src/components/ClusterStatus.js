import React from 'react';
import { Pie } from 'react-chartjs-2';

const ClusterStatus = () => {
    const data = {
        labels: ['Active Nodes', 'Inactive Nodes'],
        datasets: [{
            data: [5, 1],
            backgroundColor: [
                'rgba(75, 192, 192, 0.8)',
                'rgba(255, 99, 132, 0.8)'
            ]
        }]
    };

    return <Pie data={data} />;
};

export default ClusterStatus;