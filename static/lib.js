function load_last_position(pilot_id, elt_id) {
  fetch ("/api/json/" + pilot_id)
    .then(response => {
      return response.json();
    })
  .then(pilotInfo => {
    document.getElementById('pilot_id').innerHTML = pilotInfo.id;
    document.getElementById('pilot_latitude').innerHTML = pilotInfo.lat;
    document.getElementById('pilot_longitude').innerHTML = pilotInfo.lon;
    document.getElementById('pilot_accuracy').innerHTML = pilotInfo.accuracy;
    document.getElementById('pilot_timestamp').innerHTML = pilotInfo.ts;
  })
}
