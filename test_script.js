let accounts = [
  {
    email: 'me@example.com'
  }
];

let organizer = 'other@example.com';

let showRSVP = organizer && accounts.every(a => a.email !== organizer);
console.log(showRSVP);

organizer = 'me@example.com';
showRSVP = organizer && accounts.every(a => a.email !== organizer);
console.log(showRSVP);
