const { processData } = require('../dataProcessor');

test('processData should group data by key', () => {
    const data = [
        { key: 'a', value: 1 },
        { key: 'b', value: 2 },
        { key: 'a', value: 3 },
    ];
    const expected = [
        [1, 3],
        [2],
    ];
    expect(processData(data)).toEqual(expected);
});

// ... additional tests ...